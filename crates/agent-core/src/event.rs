#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentEvent {
    RunStarted,
    MessageStarted,
    MessageDelta(String),
    MessageCompleted(String),
    ToolCallDelta {
        id: String,
        name: String,
        arguments_delta: String,
    },
    ToolCallReceived(String),
    ToolExecutionStarted(String),
    ToolExecutionCompleted(String),
    IterationPressure {
        current: u32,
        max: u32,
        pct: u8,
    },
    RunCompleted,
}
