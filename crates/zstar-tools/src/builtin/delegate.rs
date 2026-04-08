use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};
use async_trait::async_trait;

use std::sync::Arc;

const MAX_DEPTH: u32 = 2;

pub type SubagentRunner = Arc<
    dyn Fn(
            SubagentRequest,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = SubagentResult> + Send>>
        + Send
        + Sync,
>;

#[derive(Debug, Clone)]
pub struct SubagentRequest {
    pub task: String,
    pub context: String,
    pub depth: u32,
    pub allowed_tools: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SubagentResult {
    pub final_message: String,
    pub iterations: u32,
    pub tool_calls: u32,
    pub error: Option<String>,
}

pub struct DelegateTool {
    descriptor: ToolDescriptor,
    runner: SubagentRunner,
    depth: u32,
}

impl DelegateTool {
    pub fn new(runner: SubagentRunner, depth: u32) -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "delegate",
                "Delegate a task to a sub-agent. The sub-agent runs autonomously with scoped tools and returns its result.",
            )
            .with_parameters(vec![
                ToolParameter::required("task", ParameterType::String, "Task description for the sub-agent"),
                ToolParameter::optional("context", ParameterType::String, "Additional context or instructions"),
                ToolParameter::optional(
                    "allowed_tools",
                    ParameterType::String,
                    "Comma-separated list of tool names the sub-agent may use (default: all available)",
                ),
            ]),
            runner,
            depth,
        }
    }

    pub fn depth(&self) -> u32 {
        self.depth
    }

    pub fn max_depth() -> u32 {
        MAX_DEPTH
    }
}

#[async_trait]
impl ToolExecutor for DelegateTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let task = match call.arguments.get("task").and_then(|v| v.as_str()) {
            Some(t) => t.to_string(),
            None => return ToolResult::error(&call, "delegate requires a 'task' parameter"),
        };

        let context = call
            .arguments
            .get("context")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if self.depth >= MAX_DEPTH {
            return ToolResult::error(
                &call,
                format!("delegate: max sub-agent depth ({MAX_DEPTH}) reached"),
            );
        }

        let allowed_tools = call
            .arguments
            .get("allowed_tools")
            .and_then(|v| v.as_str())
            .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
            .unwrap_or_default();

        let request = SubagentRequest {
            task,
            context,
            depth: self.depth + 1,
            allowed_tools,
        };

        let result = (self.runner)(request).await;

        match result.error {
            Some(err) => ToolResult::error(&call, format!("delegate: sub-agent failed: {err}")),
            None => {
                let output = format!(
                    "Sub-agent completed ({} iterations, {} tool calls):\n{}",
                    result.iterations, result.tool_calls, result.final_message
                );
                ToolResult::success(&call, output)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_runner(result_message: &'static str) -> SubagentRunner {
        Arc::new(move |_req| {
            let msg = result_message.to_string();
            Box::pin(async move {
                SubagentResult {
                    final_message: msg,
                    iterations: 1,
                    tool_calls: 0,
                    error: None,
                }
            })
        })
    }

    fn error_runner(err: &'static str) -> SubagentRunner {
        Arc::new(move |_req| {
            let e = err.to_string();
            Box::pin(async move {
                SubagentResult {
                    final_message: String::new(),
                    iterations: 0,
                    tool_calls: 0,
                    error: Some(e),
                }
            })
        })
    }

    #[tokio::test]
    async fn delegate_runs_subagent() {
        let tool = DelegateTool::new(simple_runner("task done"), 0);
        let call = ToolCall::new(
            "1".into(),
            "delegate".into(),
            serde_json::json!({"task": "do stuff"}),
        );
        let result = tool.execute(call).await;
        assert!(!result.is_error);
        assert!(result.output.contains("task done"));
    }

    #[tokio::test]
    async fn delegate_respects_max_depth() {
        let tool = DelegateTool::new(simple_runner("should not run"), MAX_DEPTH);
        let call = ToolCall::new(
            "1".into(),
            "delegate".into(),
            serde_json::json!({"task": "nested"}),
        );
        let result = tool.execute(call).await;
        assert!(result.is_error);
        assert!(result.output.contains("max sub-agent depth"));
    }

    #[tokio::test]
    async fn delegate_requires_task_param() {
        let tool = DelegateTool::new(simple_runner("x"), 0);
        let call = ToolCall::new("1".into(), "delegate".into(), serde_json::json!({}));
        let result = tool.execute(call).await;
        assert!(result.is_error);
        assert!(result.output.contains("task"));
    }

    #[tokio::test]
    async fn delegate_reports_subagent_error() {
        let tool = DelegateTool::new(error_runner("timeout"), 0);
        let call = ToolCall::new(
            "1".into(),
            "delegate".into(),
            serde_json::json!({"task": "fail"}),
        );
        let result = tool.execute(call).await;
        assert!(result.is_error);
        assert!(result.output.contains("sub-agent failed"));
    }

    #[tokio::test]
    async fn delegate_passes_context_and_depth() {
        let captured: Arc<std::sync::Mutex<Option<SubagentRequest>>> =
            Arc::new(std::sync::Mutex::new(None));
        let captured_clone = captured.clone();
        let runner: SubagentRunner = Arc::new(move |req| {
            let c = captured_clone.clone();
            Box::pin(async move {
                *c.lock().unwrap() = Some(req);
                SubagentResult {
                    final_message: "ok".to_string(),
                    iterations: 1,
                    tool_calls: 0,
                    error: None,
                }
            })
        });
        let tool = DelegateTool::new(runner, 1);
        let call = ToolCall::new(
            "1".into(),
            "delegate".into(),
            serde_json::json!({"task": "analyze", "context": "focus on X"}),
        );
        tool.execute(call).await;
        let req = captured.lock().unwrap().take().unwrap();
        assert_eq!(req.task, "analyze");
        assert_eq!(req.context, "focus on X");
        assert_eq!(req.depth, 2);
    }
}
