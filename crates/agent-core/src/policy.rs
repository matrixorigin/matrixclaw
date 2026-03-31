use async_trait::async_trait;
use matrixclaw_tools::{ToolCall, ToolResult};

#[derive(Debug, Clone)]
pub enum ToolPreflightDecision {
    Allow,
    Block(ToolResult),
}

#[async_trait]
pub trait ToolPreflightPolicy: Send + Sync {
    async fn before_tool_call(&self, call: &ToolCall) -> ToolPreflightDecision;
}
