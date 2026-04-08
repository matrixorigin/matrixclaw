use std::path::Path;

use zstar_agent_core::provider::Provider;
use zstar_compat_openclaw::capabilities::CapabilityDescriptor;
use zstar_compat_openclaw::http::HttpChatResponse;
use zstar_compat_openclaw::stream_adapter::ChatFrame;
use zstar_compat_openclaw::translation::OpenClawChatRequest;
use zstar_compat_openclaw::websocket::ChatWebSocketConversation;

pub use crate::ingress::{normalize_openclaw_request, OpenClawIngressMetadata};
use crate::ingress::{
    run_ingress_with_provider, run_ingress_with_provider_stream, IngressRunOutcome,
};
use crate::live_runtime::LiveRunEvent;

pub async fn openclaw_chat_http_with_provider(
    home: impl AsRef<Path>,
    model: impl Into<String>,
    request: &OpenClawChatRequest,
    provider: &mut dyn Provider,
) -> Result<HttpChatResponse, String> {
    openclaw_chat_http_with_provider_and_metadata(
        home,
        model,
        request,
        &OpenClawIngressMetadata::default(),
        provider,
    )
    .await
}

pub async fn openclaw_chat_http_with_provider_and_metadata(
    home: impl AsRef<Path>,
    model: impl Into<String>,
    request: &OpenClawChatRequest,
    metadata: &OpenClawIngressMetadata,
    provider: &mut dyn Provider,
) -> Result<HttpChatResponse, String> {
    let envelope = normalize_openclaw_request(request, metadata)?;
    let outcome = run_ingress_with_provider(home, model, &envelope, provider).await?;
    Ok(HttpChatResponse {
        conversation_id: envelope.reply.conversation_id.clone(),
        frames: frames_from_outcome(&outcome),
    })
}

pub async fn openclaw_chat_http(
    home: impl AsRef<Path>,
    model: impl Into<String>,
    request: &OpenClawChatRequest,
    metadata: &OpenClawIngressMetadata,
    provider: &mut dyn Provider,
) -> Result<HttpChatResponse, String> {
    openclaw_chat_http_with_provider_and_metadata(home, model, request, metadata, provider).await
}

pub async fn openclaw_chat_websocket_with_provider(
    home: impl AsRef<Path>,
    model: impl Into<String>,
    request: &OpenClawChatRequest,
    provider: &mut dyn Provider,
) -> Result<ChatWebSocketConversation, String> {
    openclaw_chat_websocket_with_provider_and_metadata(
        home,
        model,
        request,
        &OpenClawIngressMetadata::default(),
        provider,
    )
    .await
}

pub async fn openclaw_chat_websocket_with_provider_and_metadata(
    home: impl AsRef<Path>,
    model: impl Into<String>,
    request: &OpenClawChatRequest,
    metadata: &OpenClawIngressMetadata,
    provider: &mut dyn Provider,
) -> Result<ChatWebSocketConversation, String> {
    let envelope = normalize_openclaw_request(request, metadata)?;
    let outcome = run_ingress_with_provider(home, model, &envelope, provider).await?;
    Ok(ChatWebSocketConversation {
        capability: CapabilityDescriptor {
            agent_listing_supported: true,
            ..CapabilityDescriptor::default()
        },
        frames: frames_from_outcome(&outcome),
    })
}

pub async fn openclaw_chat_websocket(
    home: impl AsRef<Path>,
    model: impl Into<String>,
    request: &OpenClawChatRequest,
    metadata: &OpenClawIngressMetadata,
    provider: &mut dyn Provider,
) -> Result<ChatWebSocketConversation, String> {
    openclaw_chat_websocket_with_provider_and_metadata(home, model, request, metadata, provider)
        .await
}

pub async fn stream_openclaw_chat_websocket(
    home: impl AsRef<Path>,
    model: impl Into<String>,
    request: &OpenClawChatRequest,
    provider: &mut dyn Provider,
    on_frame: &mut (dyn FnMut(ChatFrame) + Send),
) -> Result<ChatWebSocketConversation, String> {
    stream_openclaw_chat_websocket_with_metadata(
        home,
        model,
        request,
        &OpenClawIngressMetadata::default(),
        provider,
        on_frame,
    )
    .await
}

pub async fn stream_openclaw_chat_websocket_with_metadata(
    home: impl AsRef<Path>,
    model: impl Into<String>,
    request: &OpenClawChatRequest,
    metadata: &OpenClawIngressMetadata,
    provider: &mut dyn Provider,
    on_frame: &mut (dyn FnMut(ChatFrame) + Send),
) -> Result<ChatWebSocketConversation, String> {
    let envelope = normalize_openclaw_request(request, metadata)?;
    let mut projector = ChatFrameProjector::default();
    let outcome =
        run_ingress_with_provider_stream(home, model, &envelope, provider, &mut |event| {
            if let Some(frame) = projector.on_event(&event) {
                on_frame(frame);
            }
        })
        .await?;

    Ok(ChatWebSocketConversation {
        capability: CapabilityDescriptor {
            agent_listing_supported: true,
            ..CapabilityDescriptor::default()
        },
        frames: projector.finish_into_frames(outcome.live_run.final_message),
    })
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

fn frames_from_outcome(outcome: &IngressRunOutcome) -> Vec<ChatFrame> {
    let mut projector = ChatFrameProjector::default();
    for event in &outcome.live_run.events {
        let _ = projector.on_event(event);
    }
    projector.finish_into_frames(outcome.live_run.final_message.clone())
}
