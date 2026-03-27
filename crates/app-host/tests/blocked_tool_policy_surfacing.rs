use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_agent_core::event::AgentEvent;
use matrixclaw_agent_core::provider::{Provider, ProviderError};
use matrixclaw_agent_core::RunRequest;
use matrixclaw_app_host::live_runtime::{session_db_path, LiveRunRequest, SessionBackedLiveRunService};
use matrixclaw_session_runtime::message_projection::{
    DurableTranscriptEntry, DurableTranscriptKind,
};
use matrixclaw_session_runtime::sqlite::SqliteStorage;
use matrixclaw_session_runtime::storage::TranscriptStore;

#[test]
fn blocked_tool_policy_surfacing() {
    let home = temp_home();
    let mut provider = ScriptedProvider::new(vec![
        "call:danger(delete_all)",
        "result:blocked: policy denied execution",
    ]);
    let service = SessionBackedLiveRunService::new(&home);

    let outcome = service
        .run_with_provider(
            "moonshotai/kimi-k2.5",
            LiveRunRequest {
                prompt: "delete everything".to_string(),
                session_id: None,
            },
            &mut provider,
        )
        .expect("live run should complete even when the tool is blocked");

    let session_path = session_db_path(&home, &outcome.session_id);
    let storage = SqliteStorage::open(&session_path).expect("open persisted session");
    let transcript = storage.load_transcript().expect("load transcript");

    assert_eq!(
        provider.stream_calls,
        2,
        "the run should continue after a blocked tool instead of crashing"
    );
    assert!(
        outcome.events.iter().any(|event| {
            event.kind == "tool_result_appended"
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

struct ScriptedProvider {
    responses: Vec<String>,
    stream_calls: usize,
}

impl ScriptedProvider {
    fn new(responses: Vec<&str>) -> Self {
        Self {
            responses: responses.into_iter().map(str::to_string).collect(),
            stream_calls: 0,
        }
    }
}

impl Provider for ScriptedProvider {
    fn complete(&mut self, _request: &RunRequest) -> Result<String, ProviderError> {
        Ok(self.responses.first().cloned().unwrap_or_default())
    }

    fn stream(
        &mut self,
        _request: &RunRequest,
        on_event: &mut dyn FnMut(AgentEvent),
    ) -> Result<String, ProviderError> {
        let response = self
            .responses
            .get(self.stream_calls)
            .cloned()
            .unwrap_or_default();
        self.stream_calls += 1;

        on_event(AgentEvent::RunStarted);
        on_event(AgentEvent::MessageStarted);
        on_event(AgentEvent::MessageDelta(response.clone()));
        on_event(AgentEvent::MessageCompleted(response.clone()));

        Ok(response)
    }
}

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = std::env::temp_dir().join(format!(
        "matrixclaw-blocked-tool-home-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&home).expect("create temp home");
    home
}
