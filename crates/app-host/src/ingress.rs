use std::path::Path;

use matrixclaw_agent_core::provider::Provider;
use matrixclaw_compat_openclaw::translation::{split_openclaw_request, OpenClawChatRequest};
use matrixclaw_session_runtime::RuntimeMessage;

use crate::http::agent_api::AgentRunRequest;
use crate::live_runtime::{
    load_or_create_session_for_request, persist_session_for_id, LiveRunEvent, LiveRunOutcome,
    LiveRunRequest, SessionBackedLiveRunService,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OpenClawIngressMetadata {
    pub sender_id: Option<String>,
    pub sender_display_name: Option<String>,
    pub channel_id: Option<String>,
    pub thread_id: Option<String>,
    pub reply_to: Option<String>,
    pub target_agent: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngressTransport {
    pub kind: String,
    pub route: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IngressSender {
    pub id: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngressConversation {
    pub session_id: String,
    pub thread_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngressPayload {
    pub prompt: String,
    pub seed_history: Vec<RuntimeMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplyRouting {
    pub conversation_id: String,
    pub channel_id: Option<String>,
    pub thread_id: Option<String>,
    pub reply_to: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngressEnvelope {
    pub transport: IngressTransport,
    pub sender: IngressSender,
    pub conversation: IngressConversation,
    pub target_agent: Option<String>,
    pub payload: IngressPayload,
    pub reply: ReplyRouting,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngressRunOutcome {
    pub reply: ReplyRouting,
    pub live_run: LiveRunOutcome,
}

impl IngressEnvelope {
    pub fn to_live_run_request(&self) -> LiveRunRequest {
        LiveRunRequest {
            prompt: self.payload.prompt.clone(),
            session_id: Some(self.conversation.session_id.clone()),
        }
    }
}

pub fn normalize_browser_request(request: &AgentRunRequest) -> Result<IngressEnvelope, String> {
    let prompt = request.prompt.trim();
    if prompt.is_empty() {
        return Err("browser ingress prompt cannot be empty".to_string());
    }

    let session_id = request
        .session_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(browser_session_id_placeholder);

    Ok(IngressEnvelope {
        transport: IngressTransport {
            kind: "browser".to_string(),
            route: "/api/agent/run".to_string(),
        },
        sender: IngressSender {
            id: Some("loopback-browser".to_string()),
            display_name: None,
        },
        conversation: IngressConversation {
            session_id: session_id.clone(),
            thread_id: None,
        },
        target_agent: None,
        payload: IngressPayload {
            prompt: prompt.to_string(),
            seed_history: Vec::new(),
        },
        reply: ReplyRouting {
            conversation_id: session_id,
            channel_id: None,
            thread_id: None,
            reply_to: None,
        },
    })
}

pub fn normalize_openclaw_request(
    request: &OpenClawChatRequest,
    metadata: &OpenClawIngressMetadata,
) -> Result<IngressEnvelope, String> {
    let (seed_history, prompt) = split_openclaw_request(request)?;
    let conversation_id = request.conversation_id.trim().to_string();
    if conversation_id.is_empty() {
        return Err("conversation_id is required".to_string());
    }

    Ok(IngressEnvelope {
        transport: IngressTransport {
            kind: "openclaw".to_string(),
            route: "/api/openclaw/chat".to_string(),
        },
        sender: IngressSender {
            id: metadata.sender_id.clone(),
            display_name: metadata.sender_display_name.clone(),
        },
        conversation: IngressConversation {
            session_id: conversation_id.clone(),
            thread_id: metadata.thread_id.clone(),
        },
        target_agent: metadata.target_agent.clone(),
        payload: IngressPayload {
            prompt,
            seed_history,
        },
        reply: ReplyRouting {
            conversation_id,
            channel_id: metadata.channel_id.clone(),
            thread_id: metadata.thread_id.clone(),
            reply_to: metadata.reply_to.clone(),
        },
    })
}

pub fn run_ingress_with_provider(
    home: impl AsRef<Path>,
    model: impl Into<String>,
    envelope: &IngressEnvelope,
    provider: &mut dyn Provider,
) -> Result<IngressRunOutcome, String> {
    run_ingress_with_provider_stream(home, model, envelope, provider, &mut |_| {})
}

pub fn run_ingress_with_provider_stream(
    home: impl AsRef<Path>,
    model: impl Into<String>,
    envelope: &IngressEnvelope,
    provider: &mut dyn Provider,
    on_event: &mut dyn FnMut(LiveRunEvent),
) -> Result<IngressRunOutcome, String> {
    let home = home.as_ref();
    let session_id = seed_session_from_ingress(home, envelope)?;
    let service = SessionBackedLiveRunService::new(home);
    let live_run = service.run_with_provider_and_queue_stream(
        model,
        LiveRunRequest {
            session_id: Some(session_id),
            ..envelope.to_live_run_request()
        },
        None,
        provider,
        on_event,
    )?;

    Ok(IngressRunOutcome {
        reply: envelope.reply.clone(),
        live_run,
    })
}

pub fn seed_session_from_ingress(
    home: &Path,
    envelope: &IngressEnvelope,
) -> Result<String, String> {
    let session_hint = normalized_session_hint(&envelope.conversation.session_id);
    let (session_id, mut session) =
        load_or_create_session_for_request(home, session_hint.as_deref())?;
    if session.history().is_empty() && !envelope.payload.seed_history.is_empty() {
        session
            .history_mut()
            .extend(envelope.payload.seed_history.iter().cloned());
        persist_session_for_id(home, &session_id, &session)?;
    }
    Ok(session_id)
}

fn normalized_session_hint(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == browser_session_id_placeholder() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn browser_session_id_placeholder() -> String {
    "__matrixclaw_browser_session__".to_string()
}
