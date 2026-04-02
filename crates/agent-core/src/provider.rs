use async_trait::async_trait;

use crate::event::AgentEvent;
use crate::{RunRequest, ToolCall};

#[derive(Debug, Clone)]
pub struct ProviderError(pub String);

impl From<&str> for ProviderError {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
}

impl ProviderResponse {
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: Some(content.into()),
            tool_calls: Vec::new(),
        }
    }

    pub fn tool_calls(calls: Vec<ToolCall>) -> Self {
        Self {
            content: None,
            tool_calls: calls,
        }
    }

    pub fn is_tool_call(&self) -> bool {
        !self.tool_calls.is_empty()
    }
}

#[async_trait]
pub trait Provider: Send + Sync {
    async fn complete(&mut self, request: &RunRequest) -> Result<ProviderResponse, ProviderError>;

    async fn stream(
        &mut self,
        request: &RunRequest,
        on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<ProviderResponse, ProviderError>;
}
