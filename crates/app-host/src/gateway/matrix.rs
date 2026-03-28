use std::path::Path;

use super::{
    GatewayInboundEvent, GatewayOutboundDelivery, GatewaySender, GatewayThread,
    OutboundDeliveryKind,
};
use crate::gateway::store::GatewaySessionStore;
use crate::ingress::IngressEnvelope;
use crate::live_runtime::LiveRunEvent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixInboundEvent {
    pub sender_id: String,
    pub sender_display_name: Option<String>,
    pub room_id: String,
    pub thread_id: Option<String>,
    pub event_id: Option<String>,
    pub target_agent: Option<String>,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixReplyRoute {
    pub room_id: String,
    pub thread: Option<GatewayThread>,
    pub reply_to: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixOutboundRoute {
    pub room_id: String,
    pub thread: Option<GatewayThread>,
    pub reply_to: Option<String>,
}

pub fn matrix_gateway_status_message() -> &'static str {
    "Matrix gateway disabled (optional; no configuration provided)"
}

pub fn normalize_matrix_inbound_event(
    home: impl AsRef<Path>,
    event: &MatrixInboundEvent,
) -> Result<IngressEnvelope, String> {
    if event.sender_id.trim().is_empty() {
        return Err("sender_id is required".to_string());
    }
    if event.room_id.trim().is_empty() {
        return Err("room_id is required".to_string());
    }
    if event.body.trim().is_empty() {
        return Err("body is required".to_string());
    }

    let store = GatewaySessionStore::load_or_default(home).map_err(|error| error.to_string())?;
    let session_id = store
        .resolve_matrix_thread(&event.room_id, event.thread_id.as_deref())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| fallback_session_id(&event.room_id, event.thread_id.as_deref()));

    Ok(GatewayInboundEvent {
        sender: GatewaySender {
            id: event.sender_id.clone(),
            display_name: event.sender_display_name.clone(),
        },
        channel_id: event.room_id.clone(),
        thread: Some(GatewayThread {
            session_id,
            thread_id: normalized_optional(event.thread_id.as_deref()),
        }),
        target_agent: event.target_agent.clone(),
        prompt: event.body.clone(),
        reply_to: normalized_optional(event.event_id.as_deref()),
    }
    .to_ingress_envelope("matrix"))
}

pub fn project_matrix_streamed_delivery(
    route: MatrixOutboundRoute,
    events: &[LiveRunEvent],
) -> Vec<GatewayOutboundDelivery> {
    let mut deliveries = Vec::new();
    let mut accumulated = String::new();

    for event in events {
        match event.kind.as_str() {
            "message_delta" => {
                let chunk = event.content.clone().unwrap_or_default();
                accumulated.push_str(&chunk);
                deliveries.push(GatewayOutboundDelivery {
                    kind: OutboundDeliveryKind::AssistantChunk,
                    channel_id: route.room_id.clone(),
                    thread: route.thread.clone(),
                    reply_to: route.reply_to.clone(),
                    body: chunk,
                });
            }
            "run_completed" => {
                deliveries.push(GatewayOutboundDelivery {
                    kind: OutboundDeliveryKind::AssistantFinal,
                    channel_id: route.room_id.clone(),
                    thread: route.thread.clone(),
                    reply_to: route.reply_to.clone(),
                    body: accumulated.clone(),
                });
            }
            _ => {}
        }
    }

    deliveries
}

fn fallback_session_id(room_id: &str, thread_id: Option<&str>) -> String {
    match normalized_optional(thread_id) {
        Some(thread_id) => format!("matrix:{}:{}", room_id.trim(), thread_id),
        None => format!("matrix:{}", room_id.trim()),
    }
}

fn normalized_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}
