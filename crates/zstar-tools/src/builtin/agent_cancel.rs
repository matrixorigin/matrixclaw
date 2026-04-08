use std::sync::Arc;

use async_trait::async_trait;

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};
use crate::subagent::SubagentTracker;

pub struct AgentCancelTool {
    descriptor: ToolDescriptor,
    tracker: Arc<SubagentTracker>,
}

impl AgentCancelTool {
    pub fn new(tracker: Arc<SubagentTracker>) -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "agent_cancel",
                "Cancel a running sub-agent by ID. The agent's status is set to cancelled.",
            )
            .with_parameters(vec![ToolParameter::required(
                "agent_id",
                ParameterType::String,
                "ID of the sub-agent to cancel",
            )]),
            tracker,
        }
    }
}

#[async_trait]
impl ToolExecutor for AgentCancelTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let agent_id = match call.arguments.get("agent_id").and_then(|v| v.as_str()) {
            Some(id) => id.to_string(),
            None => return ToolResult::error(&call, "missing required parameter: agent_id"),
        };

        let handle = self.tracker.get(&agent_id);
        if handle.is_none() {
            return ToolResult::error(&call, format!("agent_cancel: agent '{agent_id}' not found"));
        }

        let cancelled = self.tracker.cancel(&agent_id);
        if cancelled {
            ToolResult::success(&call, format!("agent '{agent_id}' cancelled"))
        } else {
            ToolResult::error(
                &call,
                format!("agent_cancel: failed to cancel agent '{agent_id}'"),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tracker() -> Arc<SubagentTracker> {
        Arc::new(SubagentTracker::new())
    }

    async fn call(tool: &AgentCancelTool, args: &str) -> ToolResult {
        let call = ToolCall::new(
            "1".into(),
            "agent_cancel".into(),
            serde_json::json!(serde_json::from_str::<serde_json::Value>(args).unwrap()),
        );
        tool.execute(call).await
    }

    #[tokio::test]
    async fn cancel_existing_agent() {
        let tracker = make_tracker();
        let id = tracker.spawn("do work".to_string());
        tracker.start(&id);

        let tool = AgentCancelTool::new(tracker.clone());
        let r = call(&tool, &format!(r#"{{"agent_id":"{id}"}}"#)).await;
        assert!(!r.is_error);
        assert!(r.output.contains("cancelled"));

        let handle = tracker.get(&id).unwrap();
        assert!(matches!(
            handle.status,
            crate::subagent::SubagentStatus::Cancelled
        ));
    }

    #[tokio::test]
    async fn cancel_nonexistent_agent() {
        let tool = AgentCancelTool::new(make_tracker());
        let r = call(&tool, r#"{"agent_id":"agent-nope"}"#).await;
        assert!(r.is_error);
        assert!(r.output.contains("not found"));
    }

    #[tokio::test]
    async fn cancel_requires_agent_id() {
        let tool = AgentCancelTool::new(make_tracker());
        let call = ToolCall::new("1".into(), "agent_cancel".into(), serde_json::json!({}));
        let r = tool.execute(call).await;
        assert!(r.is_error);
        assert!(r.output.contains("agent_id"));
    }
}
