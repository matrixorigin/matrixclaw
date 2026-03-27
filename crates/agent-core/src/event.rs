#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentEvent {
    RunStarted,
    MessageStarted,
    MessageDelta(String),
    MessageCompleted(String),
    ToolCallStarted(String),
    ToolExecutionStarted(String),
    ToolExecutionCompleted(String),
    ToolResultAppended(String),
    RunCompleted,
}
