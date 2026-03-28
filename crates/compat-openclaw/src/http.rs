use serde::Serialize;

use crate::stream_adapter::ChatFrame;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HttpChatResponse {
    pub conversation_id: String,
    pub frames: Vec<ChatFrame>,
}
