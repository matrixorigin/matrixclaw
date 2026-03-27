use crate::stream_adapter::{ChatFrame, LoopbackChatStreamAdapter};
use crate::translation::{translate_chat_request, OpenClawChatRequest};
use matrixclaw_session_runtime::ChatRuntime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpChatResponse {
    pub conversation_id: String,
    pub frames: Vec<ChatFrame>,
}

pub fn openclaw_chat_http<R>(
    request: &OpenClawChatRequest,
    runtime: &mut R,
) -> HttpChatResponse
where
    R: ChatRuntime,
{
    let mut adapter = LoopbackChatStreamAdapter::new();
    translate_chat_request(request, runtime, &mut adapter);

    HttpChatResponse {
        conversation_id: request.conversation_id.clone(),
        frames: adapter.frames().to_vec(),
    }
}
