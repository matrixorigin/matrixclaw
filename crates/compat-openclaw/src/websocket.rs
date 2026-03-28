use serde::Serialize;

use crate::auth::{validate_response, CHALLENGE_TOKEN};
use crate::capabilities::{AgentDescriptor, CapabilityDescriptor};
use crate::stream_adapter::ChatFrame;
use crate::translation::default_agents;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Frame {
    Challenge { token: String },
    Authenticated,
    AgentsList { agents: Vec<AgentDescriptor> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChatWebSocketConversation {
    pub capability: CapabilityDescriptor,
    pub frames: Vec<ChatFrame>,
}
