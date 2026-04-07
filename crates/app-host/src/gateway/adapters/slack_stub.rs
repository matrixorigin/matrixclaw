use async_trait::async_trait;

use crate::gateway::platform::{MessageGateway, MessageHandler};

pub struct SlackStubGateway {
    pub app_token: String,
    pub bot_token: String,
}

impl SlackStubGateway {
    pub fn from_config(config: &serde_json::Value) -> Result<Self, String> {
        let app_token = config
            .get("app_token")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let bot_token = config
            .get("bot_token")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        Ok(Self {
            app_token,
            bot_token,
        })
    }
}

#[async_trait]
impl MessageGateway for SlackStubGateway {
    async fn start(&self, _handler: Box<dyn MessageHandler>) -> Result<(), String> {
        Err("enable gateway-slack feature flag to use the Slack adapter".to_string())
    }

    async fn send(&self, _channel: &str, _message: &str) -> Result<(), String> {
        Err("enable gateway-slack feature flag to use the Slack adapter".to_string())
    }

    fn platform_name(&self) -> &str {
        "slack"
    }
}

#[cfg(test)]
mod tests {
    use crate::gateway::platform::{IncomingMessage, MessageHandler, OutgoingMessage};

    use super::*;

    #[tokio::test]
    async fn stub_returns_feature_flag_error() {
        let gateway = SlackStubGateway {
            app_token: "test".to_string(),
            bot_token: "test".to_string(),
        };
        let result = gateway.start(Box::new(TestHandler)).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("gateway-slack"));
    }

    #[tokio::test]
    async fn send_returns_error() {
        let gateway = SlackStubGateway {
            app_token: "test".to_string(),
            bot_token: "test".to_string(),
        };
        let result = gateway.send("C123", "hello").await;
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
