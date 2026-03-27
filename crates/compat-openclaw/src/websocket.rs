use crate::auth::{validate_response, CHALLENGE_TOKEN};
use crate::capabilities::{AgentDescriptor, CapabilityDescriptor};
use crate::stream_adapter::{ChatFrame, LoopbackChatStreamAdapter};
use crate::translation::{
    default_agents, persist_openclaw_chat_session, translate_chat_request, OpenClawChatRequest,
};
use matrixclaw_session_runtime::ChatRuntime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    Challenge { token: String },
    Authenticated,
    AgentsList { agents: Vec<AgentDescriptor> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketConversation {
    pub capability: CapabilityDescriptor,
    pub frames: Vec<Frame>,
}

pub fn openclaw_agents_list(enabled: bool) -> WebSocketConversation {
    let capability = CapabilityDescriptor {
        agent_listing_supported: enabled,
        ..CapabilityDescriptor::default()
    };

    let mut frames = Vec::new();
    if enabled {
        frames.push(Frame::Challenge {
            token: CHALLENGE_TOKEN.to_string(),
        });
        if validate_response(CHALLENGE_TOKEN) {
            frames.push(Frame::Authenticated);
            frames.push(Frame::AgentsList {
                agents: default_agents(),
            });
        }
    }

    WebSocketConversation { capability, frames }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatWebSocketConversation {
    pub capability: CapabilityDescriptor,
    pub frames: Vec<ChatFrame>,
}

pub fn openclaw_chat<R>(request: &OpenClawChatRequest, runtime: &mut R) -> ChatWebSocketConversation
where
    R: ChatRuntime,
{
    let capability = CapabilityDescriptor {
        agent_listing_supported: true,
        ..CapabilityDescriptor::default()
    };

    let mut adapter = LoopbackChatStreamAdapter::new();
    translate_chat_request(request, runtime, &mut adapter);
    persist_openclaw_chat_session(&request.conversation_id, adapter.frames())
        .expect("persist compatibility session");

    ChatWebSocketConversation {
        capability,
        frames: adapter.frames().to_vec(),
    }
}
