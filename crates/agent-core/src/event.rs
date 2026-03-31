#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentEvent {
    RunStarted,
    MessageStarted,
    MessageDelta(String),
    MessageCompleted(String),
    ToolCallReceived(String),
    ToolExecutionStarted(String),
    ToolExecutionCompleted(String),
    RunCompleted,
}
