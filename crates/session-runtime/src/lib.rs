pub mod event_sink;
pub mod error;
pub mod context_builder;
pub mod compaction;
pub mod compaction_record;
pub mod message_projection;
pub mod queue;
pub mod recovery;
pub mod retry;
pub mod sqlite;
pub mod run_controller;
pub mod session;
pub mod storage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatInputRole {
    User,
    System,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatInputMessage {
    pub role: ChatInputRole,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatRequest {
    pub messages: Vec<ChatInputMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatEvent {
    AssistantChunk(String),
    ToolCall { name: String, arguments: String },
    Warning(String),
    RetryMarker(String),
    Completed,
}

pub trait ChatRuntime {
    fn handle_chat(&mut self, request: ChatRequest) -> Vec<ChatEvent>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeMessage {
    Assistant(String),
    RuntimeSummary(String),
    ToolResult(String),
    Steering(String),
    FollowUp(String),
    Warning(String),
    RetryMarker(String),
}
