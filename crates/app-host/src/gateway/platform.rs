use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncomingMessage {
    pub platform: String,
    pub channel: String,
    pub sender: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutgoingMessage {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub platform: String,
    pub config: serde_json::Value,
}

#[async_trait]
pub trait MessageGateway: Send + Sync {
    async fn start(&self, handler: Box<dyn MessageHandler>) -> Result<(), String>;
    async fn send(&self, channel: &str, message: &str) -> Result<(), String>;
    fn platform_name(&self) -> &str;
}

pub trait MessageHandler: Send + Sync {
    fn on_message(
        &self,
        msg: IncomingMessage,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = OutgoingMessage> + Send + '_>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incoming_message_roundtrip() {
        let msg = IncomingMessage {
            platform: "matrix".to_string(),
            channel: "!room:matrix.org".to_string(),
            sender: "@user:matrix.org".to_string(),
            content: "hello".to_string(),
            thread_id: Some("thread-123".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: IncomingMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn outgoing_message_roundtrip() {
        let msg = OutgoingMessage {
            content: "response".to_string(),
            thread_id: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: OutgoingMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn incoming_message_without_optional_fields() {
        let msg = IncomingMessage {
            platform: "discord".to_string(),
            channel: "123".to_string(),
            sender: "456".to_string(),
            content: "test".to_string(),
            thread_id: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(!json.contains("thread_id"));
        let back: IncomingMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back.thread_id, None);
    }
}
