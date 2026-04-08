//! Gateway is the external communication boundary for ZStar.
//!
//! Gateways receive messages from outside systems, normalize them into
//! ingress envelopes, and project runtime replies back into channel-specific
//! deliveries. They own routing, retry, and dedupe concerns, but they do not
//! own host capabilities such as screenshots or browser automation.

pub mod adapters;
pub mod agent_bridge;
pub mod cli;
pub mod client;
pub mod config;
pub mod matrix;
pub mod platform;
pub mod runtime;
pub mod store;
pub mod transport;

use crate::ingress::{
    IngressConversation, IngressEnvelope, IngressPayload, IngressSender, IngressTransport,
    ReplyRouting,
};
use crate::live_runtime::LiveRunEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewaySender {
    pub id: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayThread {
    pub session_id: String,
    pub thread_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayInboundEvent {
    pub sender: GatewaySender,
    pub channel_id: String,
    pub thread: Option<GatewayThread>,
    pub target_agent: Option<String>,
    pub prompt: String,
    pub reply_to: Option<String>,
}

impl GatewayInboundEvent {
    pub fn to_ingress_envelope(&self, transport_kind: &str) -> IngressEnvelope {
        let session_id = self
            .thread
            .as_ref()
            .map(|thread| thread.session_id.clone())
            .unwrap_or_else(|| format!("{}:{}", transport_kind, self.channel_id));

        IngressEnvelope {
            transport: IngressTransport {
                kind: transport_kind.to_string(),
                route: format!("/gateway/{transport_kind}"),
            },
            sender: IngressSender {
                id: Some(self.sender.id.clone()),
                display_name: self.sender.display_name.clone(),
            },
            conversation: IngressConversation {
                session_id: session_id.clone(),
                thread_id: self
                    .thread
                    .as_ref()
                    .and_then(|thread| thread.thread_id.clone()),
            },
            target_agent: self.target_agent.clone(),
            payload: IngressPayload {
                prompt: self.prompt.clone(),
                seed_history: Vec::new(),
            },
            reply: ReplyRouting {
                conversation_id: session_id,
                channel_id: Some(self.channel_id.clone()),
                thread_id: self
                    .thread
                    .as_ref()
                    .and_then(|thread| thread.thread_id.clone()),
                reply_to: self.reply_to.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayReplyRoute {
    pub channel_id: String,
    pub thread: Option<GatewayThread>,
    pub reply_to: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutboundDeliveryKind {
    AssistantChunk,
    AssistantFinal,
    Progress,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayOutboundDelivery {
    pub kind: OutboundDeliveryKind,
    pub channel_id: String,
    pub thread: Option<GatewayThread>,
    pub reply_to: Option<String>,
    pub body: String,
}

pub trait GatewayAdapter {
    fn kind(&self) -> &'static str;

    fn normalize_inbound(&self, event: &GatewayInboundEvent) -> Result<IngressEnvelope, String>;

    fn project_runtime_event(
        &self,
        route: &GatewayReplyRoute,
        event: &LiveRunEvent,
    ) -> Vec<GatewayOutboundDelivery>;
}
