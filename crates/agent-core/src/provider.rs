use crate::event::AgentEvent;
use crate::RunRequest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderError(pub String);

impl From<&str> for ProviderError {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

pub trait Provider {
    fn complete(&mut self, request: &RunRequest) -> Result<String, ProviderError>;
    fn stream(
        &mut self,
        request: &RunRequest,
        on_event: &mut dyn FnMut(AgentEvent),
    ) -> Result<String, ProviderError>;
}
