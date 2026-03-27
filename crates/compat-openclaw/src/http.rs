use crate::stream_adapter::{ChatFrame, LoopbackChatStreamAdapter};
use crate::translation::{
    persist_openclaw_chat_session, translate_chat_request, OpenClawChatRequest,
};
use matrixclaw_session_runtime::ChatRuntime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpChatResponse {
    pub conversation_id: String,
    pub frames: Vec<ChatFrame>,
}

pub fn openclaw_chat_http<R>(request: &OpenClawChatRequest, runtime: &mut R) -> HttpChatResponse
where
    R: ChatRuntime,
{
    let mut adapter = LoopbackChatStreamAdapter::new();
    translate_chat_request(request, runtime, &mut adapter);
    persist_openclaw_chat_session(&request.conversation_id, adapter.frames())
        .expect("persist compatibility session");

    HttpChatResponse {
        conversation_id: request.conversation_id.clone(),
        frames: adapter.frames().to_vec(),
    }
}
