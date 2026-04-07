use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};
use async_trait::async_trait;
use delegate::SubagentRequest;

use std::sync::Arc;

use super::delegate;

pub type ParallelSubagentRunner = Arc<
    dyn Fn(
            Vec<SubagentRequest>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Vec<delegate::SubagentResult>> + Send>,
        > + Send
        + Sync,
>;

pub struct DelegateParallelTool {
    descriptor: ToolDescriptor,
    runner: ParallelSubagentRunner,
    depth: u32,
}

impl DelegateParallelTool {
    pub fn new(runner: ParallelSubagentRunner, depth: u32) -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "delegate_parallel",
                "Delegate multiple independent tasks to sub-agents running in parallel. Each task gets its own sub-agent. Returns all results aggregated.",
            )
            .with_parameters(vec![
                ToolParameter::required(
                    "tasks",
                    ParameterType::String,
                    "JSON array of task descriptions. Each element is either a string (just the task) or an object with `task` and optional `context` fields.",
                ),
                ToolParameter::optional(
                    "allowed_tools",
                    ParameterType::String,
                    "Comma-separated list of tool names all sub-agents may use (default: all available)",
                ),
            ]),
            runner,
            depth,
        }
    }
}

#[async_trait]
impl ToolExecutor for DelegateParallelTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let tasks_raw = match call.arguments.get("tasks").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => {
                return ToolResult::error(
                    &call,
                    "delegate_parallel requires a 'tasks' parameter (JSON array)",
                )
            }
        };

        let parsed: serde_json::Value = match serde_json::from_str(tasks_raw) {
            Ok(v) => v,
            Err(e) => {
                return ToolResult::error(
                    &call,
                    format!("delegate_parallel: invalid JSON in tasks: {e}"),
                )
            }
        };

        let arr = match parsed.as_array() {
            Some(a) => a,
            None => {
                return ToolResult::error(&call, "delegate_parallel: 'tasks' must be a JSON array")
            }
        };

        if arr.is_empty() {
            return ToolResult::error(&call, "delegate_parallel: tasks array is empty");
        }

        if self.depth >= delegate::DelegateTool::max_depth() {
            return ToolResult::error(
                &call,
                format!(
                    "delegate_parallel: max sub-agent depth ({}) reached",
                    delegate::DelegateTool::max_depth()
                ),
            );
        }

        let allowed_tools: Vec<String> = call
            .arguments
            .get("allowed_tools")
            .and_then(|v| v.as_str())
            .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
            .unwrap_or_default();

        let mut requests = Vec::with_capacity(arr.len());
        for item in arr {
            let (task, context) =
                match item.as_str() {
                    Some(s) => (s.to_string(), String::new()),
                    None if item.is_object() => {
                        let task = match item.get("task").and_then(|v| v.as_str()) {
                        Some(t) => t.to_string(),
                        None => return ToolResult::error(
                            &call,
                            "delegate_parallel: each task object must have a 'task' string field",
                        ),
                    };
                        let context = item
                            .get("context")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        (task, context)
                    }
                    None => return ToolResult::error(
                        &call,
                        "delegate_parallel: each task must be a string or object with 'task' field",
                    ),
                };
            requests.push(SubagentRequest {
                task,
                context,
                depth: self.depth + 1,
                allowed_tools: allowed_tools.clone(),
            });
        }

        let results = (self.runner)(requests).await;

        let mut output = String::new();
        let mut total_iterations = 0u32;
        let mut total_tool_calls = 0u32;
        let mut success_count = 0usize;
        let mut fail_count = 0usize;

        for (i, result) in results.iter().enumerate() {
            total_iterations += result.iterations;
            total_tool_calls += result.tool_calls;
            if result.error.is_some() {
                fail_count += 1;
            } else {
                success_count += 1;
            }

            output.push_str(&format!(
                "--- Task {} ({} iterations, {} tool calls) ---\n",
                i + 1,
                result.iterations,
                result.tool_calls
            ));
            if let Some(ref err) = result.error {
                output.push_str(&format!("Status: FAILED ({err})\n"));
            } else {
                output.push_str(&format!("{}\n", result.final_message));
            }
            output.push('\n');
        }

        output.push_str(&format!(
            "Summary: {success_count} succeeded, {fail_count} failed | {total_iterations} total iterations, {total_tool_calls} total tool calls"
        ));

        ToolResult::success(&call, output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use delegate::SubagentResult;

    fn simple_parallel_runner(results: Vec<SubagentResult>) -> ParallelSubagentRunner {
        Arc::new(move |_reqs| {
            let r = results.clone();
            Box::pin(async move { r.clone() })
        })
    }

    fn capturing_parallel_runner() -> (
        ParallelSubagentRunner,
        std::sync::Arc<std::sync::Mutex<Vec<SubagentRequest>>>,
    ) {
        let captured: std::sync::Arc<std::sync::Mutex<Vec<SubagentRequest>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let captured_clone = captured.clone();
        let runner: ParallelSubagentRunner = Arc::new(move |reqs| {
            let c = captured_clone.clone();
            Box::pin(async move {
                let mut g = c.lock().unwrap();
                let results: Vec<SubagentResult> = reqs
                    .iter()
                    .map(|req| {
                        g.push(req.clone());
                        SubagentResult {
                            final_message: format!("done: {}", req.task),
                            iterations: 1,
                            tool_calls: 0,
                            error: None,
                        }
                    })
                    .collect();
                results
            })
        });
        (runner, captured)
    }

    #[tokio::test]
    async fn parallel_runs_multiple_tasks() {
        let runner = simple_parallel_runner(vec![
            SubagentResult {
                final_message: "result A".into(),
                iterations: 2,
                tool_calls: 1,
                error: None,
            },
            SubagentResult {
                final_message: "result B".into(),
                iterations: 3,
                tool_calls: 2,
                error: None,
            },
            SubagentResult {
                final_message: "result C".into(),
                iterations: 1,
                tool_calls: 0,
                error: None,
            },
        ]);
        let tool = DelegateParallelTool::new(runner, 0);
        let call = ToolCall::new(
            "1".into(),
            "delegate_parallel".into(),
            serde_json::json!({"tasks": "[\"task A\", \"task B\", \"task C\"]"}),
        );
        let result = tool.execute(call).await;
        assert!(!result.is_error);
        assert!(result.output.contains("result A"));
        assert!(result.output.contains("result B"));
        assert!(result.output.contains("result C"));
        assert!(result.output.contains("3 succeeded, 0 failed"));
    }

    #[tokio::test]
    async fn parallel_handles_partial_failure() {
        let runner = simple_parallel_runner(vec![
            SubagentResult {
                final_message: "ok".into(),
                iterations: 1,
                tool_calls: 0,
                error: None,
            },
            SubagentResult {
                final_message: String::new(),
                iterations: 0,
                tool_calls: 0,
                error: Some("timeout".into()),
            },
        ]);
        let tool = DelegateParallelTool::new(runner, 0);
        let call = ToolCall::new(
            "1".into(),
            "delegate_parallel".into(),
            serde_json::json!({"tasks": "[\"do thing\", \"fail thing\"]"}),
        );
        let result = tool.execute(call).await;
        assert!(!result.is_error);
        assert!(result.output.contains("1 succeeded, 1 failed"));
        assert!(result.output.contains("FAILED"));
    }

    #[tokio::test]
    async fn parallel_empty_tasks_returns_error() {
        let runner = simple_parallel_runner(vec![]);
        let tool = DelegateParallelTool::new(runner, 0);
        let call = ToolCall::new(
            "1".into(),
            "delegate_parallel".into(),
            serde_json::json!({"tasks": "[]"}),
        );
        let result = tool.execute(call).await;
        assert!(result.is_error);
        assert!(result.output.contains("empty"));
    }

    #[tokio::test]
    async fn parallel_invalid_json_returns_error() {
        let runner = simple_parallel_runner(vec![]);
        let tool = DelegateParallelTool::new(runner, 0);
        let call = ToolCall::new(
            "1".into(),
            "delegate_parallel".into(),
            serde_json::json!({"tasks": "not json array"}),
        );
        let result = tool.execute(call).await;
        assert!(result.is_error);
        assert!(result.output.contains("invalid JSON"));
    }

    #[tokio::test]
    async fn parallel_respects_max_depth() {
        let runner = simple_parallel_runner(vec![]);
        let tool = DelegateParallelTool::new(runner, delegate::DelegateTool::max_depth());
        let call = ToolCall::new(
            "1".into(),
            "delegate_parallel".into(),
            serde_json::json!({"tasks": "[\"nested\"]"}),
        );
        let result = tool.execute(call).await;
        assert!(result.is_error);
        assert!(result.output.contains("max sub-agent depth"));
    }

    #[tokio::test]
    async fn parallel_aggregates_results() {
        let (runner, _) = capturing_parallel_runner();
        let tool = DelegateParallelTool::new(runner, 0);
        let tasks = serde_json::to_string(&serde_json::json!([
            {"task": "write tests", "context": "use tokio"},
            {"task": "add docs"}
        ]))
        .unwrap();
        let call = ToolCall::new(
            "1".into(),
            "delegate_parallel".into(),
            serde_json::json!({"tasks": tasks}),
        );
        let result = tool.execute(call).await;
        assert!(!result.is_error);
        assert!(result.output.contains("Task 1"));
        assert!(result.output.contains("Task 2"));
        assert!(result.output.contains("2 succeeded, 0 failed"));
        assert!(result.output.contains("total iterations"));
        assert!(result.output.contains("total tool calls"));
    }
}
