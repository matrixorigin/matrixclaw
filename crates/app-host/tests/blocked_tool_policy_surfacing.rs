use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use std::sync::Arc;
use zstar_agent_core::event::AgentEvent;
use zstar_agent_core::provider::{Provider, ProviderError, ProviderResponse};
use zstar_agent_core::{RunRequest, ToolCall};
use zstar_app_host::live_runtime::{session_db_path, LiveRunRequest, SessionBackedLiveRunService};
use zstar_session_runtime::message_projection::{DurableTranscriptEntry, DurableTranscriptKind};
use zstar_session_runtime::sqlite::SqliteStorage;
use zstar_session_runtime::storage::TranscriptStore;
use zstar_tools::{ToolDescriptor, ToolExecutor, ToolRegistry, ToolResult};

#[tokio::test]
async fn blocked_tool_policy_surfacing() {
    let home = temp_home();
    let registry = Arc::new(ToolRegistry::new());
    registry.register(Arc::new(DangerTool)).await;

    let mut provider = ToolCallProvider::new();
    let service = SessionBackedLiveRunService::new_from_registry(&home, registry).await;

    let outcome = service
        .run_with_provider(
            "moonshotai/kimi-k2.5",
            LiveRunRequest {
                prompt: "delete everything".to_string(),
                session_id: None,
            },
            &mut provider,
        )
        .await
        .expect("live run should complete even when the tool returns an error");

    let session_path = session_db_path(&home, &outcome.session_id);
    let storage = SqliteStorage::open(&session_path).expect("open persisted session");
    let transcript = storage.load_transcript().expect("load transcript");

    assert_eq!(
        provider.stream_calls.load(Ordering::SeqCst),
        2,
        "the run should continue after a blocked tool instead of crashing"
    );
    assert!(
        outcome.events.iter().any(|event| {
            event.kind == "tool_execution_completed"
                && event.content.as_deref() == Some("blocked: policy denied execution")
        }),
        "blocked tool result should be visible in the live event stream"
    );
    assert_eq!(
        transcript,
        vec![
            DurableTranscriptEntry {
                kind: DurableTranscriptKind::ToolResult,
                content: "result:blocked: policy denied execution".to_string(),
            },
            DurableTranscriptEntry {
                kind: DurableTranscriptKind::Assistant,
                content: "result:blocked: policy denied execution".to_string(),
            },
        ],
        "blocked tool results should persist alongside the visible assistant completion"
    );
}

struct ToolCallProvider {
    stream_calls: AtomicUsize,
}

impl ToolCallProvider {
    fn new() -> Self {
        Self {
            stream_calls: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl Provider for ToolCallProvider {
    async fn complete(&mut self, _request: &RunRequest) -> Result<ProviderResponse, ProviderError> {
        Ok(ProviderResponse::text(String::new()))
    }

    async fn stream(
        &mut self,
        _request: &RunRequest,
        _on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<ProviderResponse, ProviderError> {
        let call_num = self.stream_calls.fetch_add(1, Ordering::SeqCst);

        if call_num == 0 {
            Ok(ProviderResponse::tool_calls(vec![ToolCall::new(
                "danger_call_1".into(),
                "danger".into(),
                serde_json::json!({"target": "delete_all"}),
            )]))
        } else {
            Ok(ProviderResponse::text(
                "result:blocked: policy denied execution",
            ))
        }
    }
}

struct DangerTool;

#[async_trait]
impl ToolExecutor for DangerTool {
    fn descriptor(&self) -> &ToolDescriptor {
        static DESC: std::sync::OnceLock<ToolDescriptor> = std::sync::OnceLock::new();
        DESC.get_or_init(|| ToolDescriptor::new("danger", "dangerous operations"))
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        ToolResult::error(&call, "blocked: policy denied execution")
    }
}

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = std::env::temp_dir().join(format!(
        "zstar-blocked-tool-home-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&home).expect("create temp home");
    home
}
