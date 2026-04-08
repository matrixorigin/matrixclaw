use async_trait::async_trait;
use zstar_tools::{ToolCall, ToolResult};

use crate::approval::ApprovalChecker;

#[derive(Debug, Clone)]
pub enum ToolPreflightDecision {
    Allow,
    Block(ToolResult),
}

#[async_trait]
pub trait ToolPreflightPolicy: Send + Sync {
    async fn before_tool_call(&self, call: &ToolCall) -> ToolPreflightDecision;
}

pub struct ApprovalPolicy {
    checker: ApprovalChecker,
}

impl ApprovalPolicy {
    pub fn new(checker: ApprovalChecker) -> Self {
        Self { checker }
    }
}

#[async_trait]
impl ToolPreflightPolicy for ApprovalPolicy {
    async fn before_tool_call(&self, call: &ToolCall) -> ToolPreflightDecision {
        use crate::approval::ApprovalDecision;

        match self.checker.check(&call.name, &call.arguments) {
            ApprovalDecision::Approved => ToolPreflightDecision::Allow,
            ApprovalDecision::RequiresApproval { reason, .. } => {
                ToolPreflightDecision::Block(ToolResult::error(call, reason))
            }
        }
    }
}
