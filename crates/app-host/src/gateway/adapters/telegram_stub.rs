use async_trait::async_trait;

use crate::gateway::platform::{MessageGateway, MessageHandler};

pub struct TelegramStubGateway {
    pub token: String,
    pub allowed_chats: Vec<i64>,
}

impl TelegramStubGateway {
    pub fn from_config(config: &serde_json::Value) -> Result<Self, String> {
        let token = config
            .get("token")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let allowed_chats = config
            .get("allowed_chats")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
            .unwrap_or_default();
        Ok(Self {
            token,
            allowed_chats,
        })
    }
}

#[async_trait]
impl MessageGateway for TelegramStubGateway {
    async fn start(&self, _handler: Box<dyn MessageHandler>) -> Result<(), String> {
        Err("enable gateway-telegram feature flag to use the Telegram adapter".to_string())
    }

    async fn send(&self, _channel: &str, _message: &str) -> Result<(), String> {
        Err("enable gateway-telegram feature flag to use the Telegram adapter".to_string())
    }

    fn platform_name(&self) -> &str {
        "telegram"
    }
}

#[cfg(test)]
mod tests {
    use crate::gateway::platform::{IncomingMessage, MessageHandler, OutgoingMessage};

    use super::*;

    #[tokio::test]
    async fn stub_returns_feature_flag_error() {
        let gateway = TelegramStubGateway {
            token: "test".to_string(),
            allowed_chats: vec![],
        };
        let result = gateway.start(Box::new(TestHandler)).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("gateway-telegram"));
    }

    #[tokio::test]
    async fn send_returns_error() {
        let gateway = TelegramStubGateway {
            token: "test".to_string(),
            allowed_chats: vec![],
        };
        let result = gateway.send("123", "hello").await;
        assert!(result.is_err());
    }

    struct TestHandler;

    impl MessageHandler for TestHandler {
        fn on_message(
            &self,
            _msg: IncomingMessage,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = OutgoingMessage> + Send + '_>>
        {
            Box::pin(async {
                OutgoingMessage {
                    content: String::new(),
                    thread_id: None,
                }
            })
        }
    }
}
