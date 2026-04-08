use std::sync::Arc;

use async_trait::async_trait;

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};
use crate::subagent::{SubagentStatus, SubagentTracker};

pub struct AgentListTool {
    descriptor: ToolDescriptor,
    tracker: Arc<SubagentTracker>,
}

impl AgentListTool {
    pub fn new(tracker: Arc<SubagentTracker>) -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "agent_list",
                "List all tracked sub-agents and their current status. Optionally filter by status.",
            )
            .with_parameters(vec![ToolParameter::optional(
                "status_filter",
                ParameterType::String,
                "Filter agents by status: spawned, running, completed, failed, cancelled",
            )]),
            tracker,
        }
    }
}

#[async_trait]
impl ToolExecutor for AgentListTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let agents = self.tracker.list();

        let filtered: Vec<_> =
            if let Some(filter) = call.arguments.get("status_filter").and_then(|v| v.as_str()) {
                let normalized = filter.to_lowercase();
                agents
                    .into_iter()
                    .filter(|a| match &a.status {
                        SubagentStatus::Spawned => normalized == "spawned",
                        SubagentStatus::Running => normalized == "running",
                        SubagentStatus::Completed => normalized == "completed",
                        SubagentStatus::Failed(_) => normalized == "failed",
                        SubagentStatus::Cancelled => normalized == "cancelled",
                    })
                    .collect()
            } else {
                agents
            };

        if filtered.is_empty() {
            return ToolResult::success(&call, "(no sub-agents)");
        }

        let mut output = String::new();
        for handle in &filtered {
            let duration = handle
                .duration()
                .map(|d| format!("{:.1}s", d.as_secs_f64()))
                .unwrap_or_else(|| "running".to_string());
            output.push_str(&format!(
                "{} | {} | {} | {}\n",
                handle.id, handle.task, handle.status, duration
            ));
        }
        output.push_str(&format!("\n{} agent(s)", filtered.len()));

        ToolResult::success(&call, output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subagent::SubagentResult;

    fn make_tracker() -> Arc<SubagentTracker> {
        Arc::new(SubagentTracker::new())
    }

    async fn call(tool: &AgentListTool, args: &str) -> ToolResult {
        let call = ToolCall::new(
            "1".into(),
            "agent_list".into(),
            serde_json::json!(serde_json::from_str::<serde_json::Value>(args).unwrap()),
        );
        tool.execute(call).await
    }

    #[tokio::test]
    async fn list_empty() {
        let tool = AgentListTool::new(make_tracker());
        let r = call(&tool, r#"{}"#).await;
        assert!(!r.is_error);
        assert_eq!(r.output, "(no sub-agents)");
    }

    #[tokio::test]
    async fn list_with_agents() {
        let tracker = make_tracker();
        let id = tracker.spawn("task A".to_string());
        tracker.start(&id);
        tracker.spawn("task B".to_string());

        let tool = AgentListTool::new(tracker);
        let r = call(&tool, r#"{}"#).await;
        assert!(!r.is_error);
        assert!(r.output.contains("task A"));
        assert!(r.output.contains("task B"));
        assert!(r.output.contains("2 agent(s)"));
    }

    #[tokio::test]
    async fn list_with_status_filter() {
        let tracker = make_tracker();
        let id = tracker.spawn("running task".to_string());
        tracker.start(&id);
        let id2 = tracker.spawn("completed task".to_string());
        tracker.complete(
            &id2,
            SubagentResult {
                final_message: "ok".to_string(),
                iterations: 1,
                tool_calls: 0,
                error: None,
            },
        );

        let tool = AgentListTool::new(tracker);
        let r = call(&tool, r#"{"status_filter":"running"}"#).await;
        assert!(!r.is_error);
        assert!(r.output.contains("running task"));
        assert!(!r.output.contains("completed task"));
        assert!(r.output.contains("1 agent(s)"));
    }

    #[tokio::test]
    async fn list_filter_no_match() {
        let tracker = make_tracker();
        tracker.spawn("some task".to_string());

        let tool = AgentListTool::new(tracker);
        let r = call(&tool, r#"{"status_filter":"completed"}"#).await;
        assert!(!r.is_error);
        assert_eq!(r.output, "(no sub-agents)");
    }
}
