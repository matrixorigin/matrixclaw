use crate::capabilities::AgentDescriptor;
use crate::stream_adapter::{project_runtime_event, ChatStreamAdapter};
use matrixclaw_session_runtime::{
    ChatInputMessage, ChatInputRole, ChatRequest, ChatRuntime, RuntimeMessage,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenClawChatRole {
    User,
    System,
    Tool,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenClawChatMessage {
    pub role: OpenClawChatRole,
    pub content: String,
}

impl OpenClawChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: OpenClawChatRole::User,
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: OpenClawChatRole::System,
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenClawChatRequest {
    pub conversation_id: String,
    pub messages: Vec<OpenClawChatMessage>,
}

impl OpenClawChatRequest {
    pub fn new(conversation_id: impl Into<String>, messages: Vec<OpenClawChatMessage>) -> Self {
        Self {
            conversation_id: conversation_id.into(),
            messages,
        }
    }
}

pub fn translate_chat_request<R, A>(
    request: &OpenClawChatRequest,
    runtime: &mut R,
    adapter: &mut A,
) -> ChatRequest
where
    R: ChatRuntime,
    A: ChatStreamAdapter,
{
    let runtime_messages = request
        .messages
        .iter()
        .filter_map(|message| translate_chat_message(message))
        .collect();

    let runtime_request = ChatRequest {
        messages: runtime_messages,
    };

    let events = runtime.handle_chat(runtime_request.clone());
    for event in events {
        if let Some(frame) = project_runtime_event(&event) {
            adapter.emit(frame);
        }
    }

    runtime_request
}

pub fn translate_chat_message(message: &OpenClawChatMessage) -> Option<ChatInputMessage> {
    let role = match message.role {
        OpenClawChatRole::User => ChatInputRole::User,
        OpenClawChatRole::System => ChatInputRole::System,
        OpenClawChatRole::Tool => ChatInputRole::Tool,
        OpenClawChatRole::Assistant => return None,
    };

    Some(ChatInputMessage {
        role,
        content: message.content.clone(),
    })
}

pub fn default_agents() -> Vec<AgentDescriptor> {
    vec![AgentDescriptor {
        id: "default".to_string(),
        name: "default".to_string(),
    }]
}

pub fn split_openclaw_request(
    request: &OpenClawChatRequest,
) -> Result<(Vec<RuntimeMessage>, String), String> {
    let Some((last, history_messages)) = request.messages.split_last() else {
        return Err("OpenClaw chat request must include at least one user message".to_string());
    };

    if last.role != OpenClawChatRole::User {
        return Err(
            "OpenClaw live chat requests must end with a user message for the shared runtime"
                .to_string(),
        );
    }

    if last.content.trim().is_empty() {
        return Err("OpenClaw live chat prompt cannot be empty".to_string());
    }

    let history = history_messages
        .iter()
        .map(runtime_message_from_openclaw_message)
        .collect();
    Ok((history, last.content.clone()))
}

fn runtime_message_from_openclaw_message(message: &OpenClawChatMessage) -> RuntimeMessage {
    match message.role {
        OpenClawChatRole::User => RuntimeMessage::User(message.content.clone()),
        OpenClawChatRole::System => RuntimeMessage::RuntimeSummary(message.content.clone()),
        OpenClawChatRole::Tool => RuntimeMessage::ToolResult(message.content.clone()),
        OpenClawChatRole::Assistant => RuntimeMessage::Assistant(message.content.clone()),
    }
}
