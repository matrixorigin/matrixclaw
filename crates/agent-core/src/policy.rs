use crate::tool::{BlockedToolResult, ToolExecutionRequest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolPreflightDecision {
    Allow,
    Block(BlockedToolResult),
}

pub trait ToolPreflightPolicy {
    fn before_tool_call(&mut self, request: &ToolExecutionRequest) -> ToolPreflightDecision;
}
