use std::path::Path;

use matrixclaw_agent_core::provider::Provider;
use matrixclaw_compat_openclaw::capabilities::CapabilityDescriptor;
use matrixclaw_compat_openclaw::http::HttpChatResponse;
use matrixclaw_compat_openclaw::stream_adapter::ChatFrame;
use matrixclaw_compat_openclaw::translation::{split_openclaw_request, OpenClawChatRequest};
use matrixclaw_compat_openclaw::websocket::ChatWebSocketConversation;
use matrixclaw_session_runtime::RuntimeMessage;

use crate::live_runtime::{
    load_or_create_session_for_request, persist_session_for_id, LiveRunOutcome, LiveRunRequest,
    SessionBackedLiveRunService,
};

pub fn openclaw_chat_http_with_provider(
    home: impl AsRef<Path>,
    model: impl Into<String>,
    request: &OpenClawChatRequest,
    provider: &mut dyn Provider,
) -> Result<HttpChatResponse, String> {
    let outcome = run_openclaw_with_provider(home, model, request, provider)?;
    Ok(HttpChatResponse {
        conversation_id: outcome.session_id.clone(),
        frames: frames_from_outcome(&outcome),
    })
}

pub fn openclaw_chat_http(
    home: impl AsRef<Path>,
    model: impl Into<String>,
    request: &OpenClawChatRequest,
    provider: &mut dyn Provider,
) -> Result<HttpChatResponse, String> {
    openclaw_chat_http_with_provider(home, model, request, provider)
}

pub fn openclaw_chat_websocket_with_provider(
    home: impl AsRef<Path>,
    model: impl Into<String>,
    request: &OpenClawChatRequest,
    provider: &mut dyn Provider,
) -> Result<ChatWebSocketConversation, String> {
    let outcome = run_openclaw_with_provider(home, model, request, provider)?;
    Ok(ChatWebSocketConversation {
        capability: CapabilityDescriptor {
            agent_listing_supported: true,
            ..CapabilityDescriptor::default()
        },
        frames: frames_from_outcome(&outcome),
    })
}

pub fn openclaw_chat_websocket(
    home: impl AsRef<Path>,
    model: impl Into<String>,
    request: &OpenClawChatRequest,
    provider: &mut dyn Provider,
) -> Result<ChatWebSocketConversation, String> {
    openclaw_chat_websocket_with_provider(home, model, request, provider)
}

fn run_openclaw_with_provider(
    home: impl AsRef<Path>,
    model: impl Into<String>,
    request: &OpenClawChatRequest,
    provider: &mut dyn Provider,
) -> Result<LiveRunOutcome, String> {
    let home = home.as_ref();
    let (seed_history, prompt) = split_openclaw_request(request)?;
    let session_id =
        seed_session_from_openclaw_request(home, request.conversation_id.as_str(), seed_history)?;
    let service = SessionBackedLiveRunService::new(home);
    service.run_with_provider(
        model,
        LiveRunRequest {
            prompt,
            session_id: Some(session_id),
        },
        provider,
    )
}

fn seed_session_from_openclaw_request(
    home: &Path,
    conversation_id: &str,
    seed_history: Vec<RuntimeMessage>,
) -> Result<String, String> {
    let (session_id, mut session) =
        load_or_create_session_for_request(home, Some(conversation_id))?;
    if session.history().is_empty() && !seed_history.is_empty() {
        session.history_mut().extend(seed_history);
        persist_session_for_id(home, &session_id, &session)?;
    }
    Ok(session_id)
}

fn frames_from_outcome(outcome: &LiveRunOutcome) -> Vec<ChatFrame> {
    let mut frames = Vec::new();
    let mut emitted_assistant_content = false;

    for event in &outcome.events {
        match event.kind.as_str() {
            "message_delta" => {
                if let Some(content) = &event.content {
                    emitted_assistant_content = true;
                    frames.push(ChatFrame::AssistantChunk {
                        content: content.clone(),
                    });
                }
            }
            "tool_call_started" => {
                if let Some(name) = &event.content {
                    frames.push(ChatFrame::ToolCall {
                        name: name.clone(),
                        arguments: String::new(),
                    });
                }
            }
            _ => {}
        }
    }

    if !emitted_assistant_content && !outcome.final_message.is_empty() {
        frames.push(ChatFrame::AssistantChunk {
            content: outcome.final_message.clone(),
        });
    }
    frames.push(ChatFrame::Completed);
    frames
}
