use matrixclaw_compat_openclaw::stream_adapter::{ChatFrame, LoopbackChatStreamAdapter};
use matrixclaw_compat_openclaw::translation::{
    translate_chat_request, OpenClawChatMessage, OpenClawChatRequest,
};
use matrixclaw_session_runtime::{
    ChatEvent, ChatInputMessage, ChatInputRole, ChatRequest, ChatRuntime,
};

#[derive(Default)]
struct RecordingRuntime {
    seen_requests: Vec<ChatRequest>,
}

impl ChatRuntime for RecordingRuntime {
    fn handle_chat(&mut self, request: ChatRequest) -> Vec<ChatEvent> {
        self.seen_requests.push(request);
        vec![
            ChatEvent::AssistantChunk("internal answer".to_string()),
            ChatEvent::Completed,
        ]
    }
}

#[test]
fn chat_request_translation() {
    let request = OpenClawChatRequest::new(
        "conversation-1",
        vec![
            OpenClawChatMessage::user("hello from the client"),
            OpenClawChatMessage::system("keep the runtime protocol-agnostic"),
        ],
    );

    let mut runtime = RecordingRuntime::default();
    let mut stream = LoopbackChatStreamAdapter::new();

    let runtime_request = translate_chat_request(&request, &mut runtime, &mut stream);

    assert_eq!(
        (runtime.seen_requests, stream.frames().to_vec()),
        (
            vec![ChatRequest {
                messages: vec![
                    ChatInputMessage {
                        role: ChatInputRole::User,
                        content: "hello from the client".to_string(),
                    },
                    ChatInputMessage {
                        role: ChatInputRole::System,
                        content: "keep the runtime protocol-agnostic".to_string(),
                    },
                ],
            }],
            vec![
                ChatFrame::AssistantChunk {
                    content: "internal answer".to_string(),
                },
                ChatFrame::Completed,
            ],
        ),
        "OpenClaw chat requests should translate into internal session-runtime messages and stream back as compatibility frames without exposing protocol types to the core",
    );

    assert_eq!(
        runtime_request.messages.len(),
        2,
        "translated runtime request should preserve both user-facing and system context messages"
    );
}
