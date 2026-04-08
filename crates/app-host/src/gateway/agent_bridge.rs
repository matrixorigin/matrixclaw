use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use zstar_provider::fallback::FallbackProvider;
use zstar_provider::registry::ProviderRegistry;

use super::platform::{IncomingMessage, MessageHandler, OutgoingMessage};
use crate::live_runtime::{LiveRunRequest, SessionBackedLiveRunService};
use crate::paths;

pub struct AgentBridge {
    provider_registry: Arc<ProviderRegistry>,
    fallback_chain: Vec<String>,
    model: String,
    max_message_length: Option<usize>,
    sessions: Arc<Mutex<HashMap<String, String>>>,
}

impl AgentBridge {
    pub fn new(
        provider_registry: Arc<ProviderRegistry>,
        fallback_chain: Vec<String>,
        model: String,
        max_message_length: Option<usize>,
    ) -> Self {
        Self {
            provider_registry,
            fallback_chain,
            model,
            max_message_length,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn session_id_for(
        &self,
        platform: &str,
        channel: &str,
        thread_id: Option<&str>,
    ) -> String {
        let key = match thread_id {
            Some(tid) => format!("{platform}:{channel}:{tid}"),
            None => format!("{platform}:{channel}"),
        };
        let mut sessions = self.sessions.lock().await;
        sessions
            .entry(key.clone())
            .or_insert_with(|| format!("gateway-{key}"))
            .clone()
    }

    fn truncate_message(&self, content: &str) -> String {
        match self.max_message_length {
            Some(max) => {
                if content.len() <= max {
                    content.to_string()
                } else {
                    let truncated: String = content.chars().take(max).collect();
                    format!("{truncated}...")
                }
            }
            None => content.to_string(),
        }
    }
}

impl MessageHandler for AgentBridge {
    fn on_message(
        &self,
        msg: IncomingMessage,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = OutgoingMessage> + Send + '_>> {
        Box::pin(async move {
            let session_id = self
                .session_id_for(&msg.platform, &msg.channel, msg.thread_id.as_deref())
                .await;

            let home = paths::home_dir();
            let service = SessionBackedLiveRunService::new(&home).await;

            let request = LiveRunRequest {
                prompt: msg.content.clone(),
                session_id: Some(session_id),
            };

            let mut provider =
                FallbackProvider::new(self.provider_registry.clone(), self.fallback_chain.clone());

            let mut on_event = |_event: crate::live_runtime::LiveRunEvent| {};

            let result = service
                .run_with_provider_and_queue_stream(
                    &self.model,
                    request,
                    None,
                    &mut provider,
                    &mut on_event,
                )
                .await;

            let content = match result {
                Ok(outcome) => self.truncate_message(&outcome.final_message),
                Err(e) => format!("Error: {e}"),
            };

            OutgoingMessage {
                content,
                thread_id: msg.thread_id.clone(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn same_channel_gets_same_session() {
        let registry = Arc::new(ProviderRegistry::new());
        let bridge = AgentBridge::new(registry, vec![], "test-model".to_string(), None);

        let sid1 = bridge
            .session_id_for("matrix", "!room:matrix.org", None)
            .await;
        let sid2 = bridge
            .session_id_for("matrix", "!room:matrix.org", None)
            .await;
        assert_eq!(sid1, sid2);
    }

    #[tokio::test]
    async fn different_channels_get_different_sessions() {
        let registry = Arc::new(ProviderRegistry::new());
        let bridge = AgentBridge::new(registry, vec![], "test-model".to_string(), None);

        let sid1 = bridge
            .session_id_for("matrix", "!room1:matrix.org", None)
            .await;
        let sid2 = bridge
            .session_id_for("matrix", "!room2:matrix.org", None)
            .await;
        assert_ne!(sid1, sid2);
    }

    #[tokio::test]
    async fn thread_id_affects_session() {
        let registry = Arc::new(ProviderRegistry::new());
        let bridge = AgentBridge::new(registry, vec![], "test-model".to_string(), None);

        let sid1 = bridge
            .session_id_for("matrix", "!room:matrix.org", None)
            .await;
        let sid2 = bridge
            .session_id_for("matrix", "!room:matrix.org", Some("thread-1"))
            .await;
        assert_ne!(sid1, sid2);
    }

    #[test]
    fn max_message_length_truncation() {
        let registry = Arc::new(ProviderRegistry::new());
        let bridge = AgentBridge::new(registry, vec![], "test-model".to_string(), Some(10));

        let short = bridge.truncate_message("hello");
        assert_eq!(short, "hello");

        let long = bridge.truncate_message("this is a very long message that exceeds the limit");
        assert!(long.len() <= 13);
        assert!(long.ends_with("..."));
    }

    #[test]
    fn no_truncation_when_no_limit() {
        let registry = Arc::new(ProviderRegistry::new());
        let bridge = AgentBridge::new(registry, vec![], "test-model".to_string(), None);

        let msg = "a very long message".repeat(100);
        let result = bridge.truncate_message(&msg);
        assert_eq!(result, msg);
    }
}
