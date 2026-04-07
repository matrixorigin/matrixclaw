use async_trait::async_trait;

use crate::gateway::platform::{MessageGateway, MessageHandler};

pub struct MatrixStubGateway {
    pub homeserver: String,
    pub access_token: String,
    pub rooms: Vec<String>,
}

impl MatrixStubGateway {
    pub fn from_config(config: &serde_json::Value) -> Result<Self, String> {
        let homeserver = config
            .get("homeserver")
            .and_then(|v| v.as_str())
            .unwrap_or("https://matrix.org")
            .to_string();
        let access_token = config
            .get("access_token")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let rooms = config
            .get("rooms")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        Ok(Self {
            homeserver,
            access_token,
            rooms,
        })
    }
}

#[async_trait]
impl MessageGateway for MatrixStubGateway {
    async fn start(&self, _handler: Box<dyn MessageHandler>) -> Result<(), String> {
        Err("enable gateway-matrix feature flag to use the Matrix adapter".to_string())
    }

    async fn send(&self, _channel: &str, _message: &str) -> Result<(), String> {
        Err("enable gateway-matrix feature flag to use the Matrix adapter".to_string())
    }

    fn platform_name(&self) -> &str {
        "matrix"
    }
}

#[cfg(test)]
mod tests {
    use crate::gateway::platform::{IncomingMessage, MessageHandler, OutgoingMessage};

    use super::*;

    #[test]
    fn stub_returns_feature_flag_error() {
        let config = serde_json::json!({
            "homeserver": "https://matrix.org",
            "access_token": "test",
            "rooms": ["!room:matrix.org"]
        });
        let gateway = MatrixStubGateway::from_config(&config).unwrap();
        assert_eq!(gateway.platform_name(), "matrix");
        assert_eq!(gateway.homeserver, "https://matrix.org");
        assert_eq!(gateway.rooms.len(), 1);
    }

    #[tokio::test]
    async fn start_returns_error() {
        let gateway = MatrixStubGateway {
            homeserver: "https://matrix.org".to_string(),
            access_token: "test".to_string(),
            rooms: vec![],
        };
        let result = gateway.start(Box::new(TestHandler)).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("gateway-matrix"));
    }

    #[tokio::test]
    async fn send_returns_error() {
        let gateway = MatrixStubGateway {
            homeserver: "https://matrix.org".to_string(),
            access_token: "test".to_string(),
            rooms: vec![],
        };
        let result = gateway.send("!room:matrix.org", "hello").await;
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
