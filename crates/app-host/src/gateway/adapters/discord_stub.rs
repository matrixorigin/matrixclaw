use async_trait::async_trait;

use crate::gateway::platform::{MessageGateway, MessageHandler};

pub struct DiscordStubGateway {
    pub token: String,
    pub channels: Vec<String>,
}

impl DiscordStubGateway {
    pub fn from_config(config: &serde_json::Value) -> Result<Self, String> {
        let token = config
            .get("token")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let channels = config
            .get("channels")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        Ok(Self { token, channels })
    }
}

#[async_trait]
impl MessageGateway for DiscordStubGateway {
    async fn start(&self, _handler: Box<dyn MessageHandler>) -> Result<(), String> {
        Err("enable gateway-discord feature flag to use the Discord adapter".to_string())
    }

    async fn send(&self, _channel: &str, _message: &str) -> Result<(), String> {
        Err("enable gateway-discord feature flag to use the Discord adapter".to_string())
    }

    fn platform_name(&self) -> &str {
        "discord"
    }
}

#[cfg(test)]
mod tests {
    use crate::gateway::platform::{IncomingMessage, MessageHandler, OutgoingMessage};

    use super::*;

    #[tokio::test]
    async fn stub_returns_feature_flag_error() {
        let gateway = DiscordStubGateway {
            token: "test".to_string(),
            channels: vec![],
        };
        let result = gateway.start(Box::new(TestHandler)).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("gateway-discord"));
    }

    #[tokio::test]
    async fn send_returns_error() {
        let gateway = DiscordStubGateway {
            token: "test".to_string(),
            channels: vec![],
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
