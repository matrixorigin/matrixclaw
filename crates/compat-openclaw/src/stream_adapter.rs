use serde::Serialize;

use matrixclaw_session_runtime::ChatEvent;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatFrame {
    AssistantChunk { content: String },
    ToolCall { name: String, arguments: String },
    Warning { content: String },
    RetryMarker { content: String },
    Completed,
}

pub trait ChatStreamAdapter {
    fn emit(&mut self, frame: ChatFrame);
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoopbackChatStreamAdapter {
    frames: Vec<ChatFrame>,
}

impl LoopbackChatStreamAdapter {
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }

    pub fn frames(&self) -> &[ChatFrame] {
        &self.frames
    }
}

impl ChatStreamAdapter for LoopbackChatStreamAdapter {
    fn emit(&mut self, frame: ChatFrame) {
        self.frames.push(frame);
    }
}

pub fn project_runtime_event(event: &ChatEvent) -> Option<ChatFrame> {
    match event {
        ChatEvent::AssistantChunk(content) => Some(ChatFrame::AssistantChunk {
            content: content.clone(),
        }),
        ChatEvent::ToolCall { name, arguments } => Some(ChatFrame::ToolCall {
            name: name.clone(),
            arguments: arguments.clone(),
        }),
        ChatEvent::Warning(content) => Some(ChatFrame::Warning {
            content: content.clone(),
        }),
        ChatEvent::RetryMarker(content) => Some(ChatFrame::RetryMarker {
            content: content.clone(),
        }),
        ChatEvent::Completed => Some(ChatFrame::Completed),
    }
}
