use std::path::Path;

use matrixclaw_agent_core::provider::Provider;
use matrixclaw_compat_openclaw::capabilities::CapabilityDescriptor;
use matrixclaw_compat_openclaw::http::HttpChatResponse;
use matrixclaw_compat_openclaw::stream_adapter::ChatFrame;
use matrixclaw_compat_openclaw::translation::{split_openclaw_request, OpenClawChatRequest};
use matrixclaw_compat_openclaw::websocket::ChatWebSocketConversation;
use matrixclaw_session_runtime::RuntimeMessage;

use crate::live_runtime::{
    load_or_create_session_for_request, persist_session_for_id, LiveRunEvent, LiveRunOutcome,
    LiveRunRequest, SessionBackedLiveRunService,
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

pub fn stream_openclaw_chat_websocket(
    home: impl AsRef<Path>,
    model: impl Into<String>,
    request: &OpenClawChatRequest,
    provider: &mut dyn Provider,
    on_frame: &mut dyn FnMut(ChatFrame),
) -> Result<ChatWebSocketConversation, String> {
    let home = home.as_ref();
    let model = model.into();
    let (seed_history, prompt) = split_openclaw_request(request)?;
    let session_id =
        seed_session_from_openclaw_request(home, request.conversation_id.as_str(), seed_history)?;
    let service = SessionBackedLiveRunService::new(home);
    let mut projector = ChatFrameProjector::default();
    let outcome = service.run_with_provider_and_queue_stream(
        model,
        LiveRunRequest {
            prompt,
            session_id: Some(session_id),
        },
        None,
        provider,
        &mut |event| {
            if let Some(frame) = projector.on_event(&event) {
                on_frame(frame);
            }
        },
    )?;

    Ok(ChatWebSocketConversation {
        capability: CapabilityDescriptor {
            agent_listing_supported: true,
            ..CapabilityDescriptor::default()
        },
        frames: projector.finish_into_frames(outcome.final_message),
    })
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

#[derive(Debug, Default)]
struct ChatFrameProjector {
    frames: Vec<ChatFrame>,
    emitted_assistant_content: bool,
}

impl ChatFrameProjector {
    fn on_event(&mut self, event: &LiveRunEvent) -> Option<ChatFrame> {
        match event.kind.as_str() {
            "message_started" => {
                self.emitted_assistant_content = false;
                None
            }
            "message_delta" => {
                let content = event.content.as_ref()?.clone();
                self.emitted_assistant_content = true;
                let frame = ChatFrame::AssistantChunk { content };
                self.frames.push(frame.clone());
                Some(frame)
            }
            "message_completed" => {
                if self.emitted_assistant_content {
                    None
                } else {
                    let content = event.content.as_ref()?.clone();
                    if content.is_empty() {
                        None
                    } else {
                        self.emitted_assistant_content = true;
                        let frame = ChatFrame::AssistantChunk { content };
                        self.frames.push(frame.clone());
                        Some(frame)
                    }
                }
            }
            "tool_call_started" => {
                let name = event.content.as_ref()?.clone();
                let frame = ChatFrame::ToolCall {
                    name,
                    arguments: String::new(),
                };
                self.frames.push(frame.clone());
                Some(frame)
            }
            "run_completed" => {
                let frame = ChatFrame::Completed;
                self.frames.push(frame.clone());
                Some(frame)
            }
            _ => None,
        }
    }

    fn finish_into_frames(mut self, final_message: String) -> Vec<ChatFrame> {
        if !self.emitted_assistant_content && !final_message.is_empty() {
            self.frames.push(ChatFrame::AssistantChunk {
                content: final_message,
            });
        }
        if !matches!(self.frames.last(), Some(ChatFrame::Completed)) {
            self.frames.push(ChatFrame::Completed);
        }
        self.frames
    }
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
    let mut projector = ChatFrameProjector::default();
    for event in &outcome.events {
        let _ = projector.on_event(event);
    }
    projector.finish_into_frames(outcome.final_message.clone())
}
