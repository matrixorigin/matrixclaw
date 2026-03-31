use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use matrixclaw_agent_core::event::AgentEvent;
use matrixclaw_agent_core::provider::{Provider, ProviderError, ProviderResponse};
use matrixclaw_agent_core::r#loop::run_prompt_with_policy;
use matrixclaw_agent_core::{RunRequest, ToolCall};
use matrixclaw_app_host::live_runtime::{
    session_db_path, LiveRunEvent, LiveRunRequest, SessionBackedLiveRunService,
};
use matrixclaw_session_runtime::message_projection::{
    DurableTranscriptEntry, DurableTranscriptKind,
};
use matrixclaw_session_runtime::sqlite::SqliteStorage;
use matrixclaw_session_runtime::storage::TranscriptStore;
use matrixclaw_tools::{ToolDescriptor, ToolExecutor, ToolRegistry, ToolResult};
use std::sync::Arc;

#[tokio::test]
async fn live_tool_execution() {
    let home = temp_home();

    let mut expected_provider = ScriptedProvider::new(vec![
        ScriptedResponse::ToolCall(ToolCall::new(
            "add_call_1".into(),
            "add".into(),
            serde_json::json!({"a": 2, "b": 3}),
        )),
        ScriptedResponse::Text("result:5".into()),
    ]);
    let registry = ToolRegistry::new();
    registry.register(Arc::new(AddTool)).await;
    let expected_trace = run_prompt_with_policy(
        &mut expected_provider,
        &RunRequest::new("add 2 and 3"),
        &registry,
        None,
        &mut |_| {},
    )
    .await
    .expect("the core loop should define the live tool contract");

    let expected_final_message = expected_trace.result.final_message.clone();
    assert_eq!(
        expected_final_message, "result:5",
        "the agent-core loop should continue after the tool result is available"
    );

    let mut live_provider = ScriptedProvider::new(vec![
        ScriptedResponse::ToolCall(ToolCall::new(
            "add_call_1".into(),
            "add".into(),
            serde_json::json!({"a": 2, "b": 3}),
        )),
        ScriptedResponse::Text("result:5".into()),
    ]);
    let registry = Arc::new(ToolRegistry::new());
    registry.register(Arc::new(AddTool)).await;
    let service = SessionBackedLiveRunService::new_from_registry(&home, registry).await;
    let outcome = service
        .run_with_provider(
            "moonshotai/kimi-k2.5",
            LiveRunRequest {
                prompt: "add 2 and 3".to_string(),
                session_id: None,
            },
            &mut live_provider,
        )
        .await
        .expect("live run should produce an outcome");

    let session_path = session_db_path(&home, &outcome.session_id);
    let storage = SqliteStorage::open(&session_path).expect("open persisted session");
    let transcript = storage
        .load_transcript()
        .expect("load persisted transcript");

    let expected_events = vec![
        LiveRunEvent {
            sequence: 0,
            kind: "run_started".to_string(),
            content: None,
        },
        LiveRunEvent {
            sequence: 1,
            kind: "message_started".to_string(),
            content: None,
        },
        LiveRunEvent {
            sequence: 2,
            kind: "message_delta".to_string(),
            content: Some("call:add({\"a\":2,\"b\":3})".to_string()),
        },
        LiveRunEvent {
            sequence: 3,
            kind: "message_completed".to_string(),
            content: Some(String::new()),
        },
        LiveRunEvent {
            sequence: 4,
            kind: "tool_call_started".to_string(),
            content: Some("add".to_string()),
        },
        LiveRunEvent {
            sequence: 5,
            kind: "tool_execution_started".to_string(),
            content: Some("add".to_string()),
        },
        LiveRunEvent {
            sequence: 6,
            kind: "tool_execution_completed".to_string(),
            content: Some("5".to_string()),
        },
        LiveRunEvent {
            sequence: 7,
            kind: "message_started".to_string(),
            content: None,
        },
        LiveRunEvent {
            sequence: 8,
            kind: "message_delta".to_string(),
            content: Some("result:5".to_string()),
        },
        LiveRunEvent {
            sequence: 9,
            kind: "message_completed".to_string(),
            content: Some("result:5".to_string()),
        },
        LiveRunEvent {
            sequence: 10,
            kind: "run_completed".to_string(),
            content: None,
        },
    ];

    let expected_transcript = vec![
        DurableTranscriptEntry {
            kind: DurableTranscriptKind::ToolResult,
            content: "result:5".to_string(),
        },
        DurableTranscriptEntry {
            kind: DurableTranscriptKind::Assistant,
            content: "result:5".to_string(),
        },
    ];

    assert_eq!(
        (
            live_provider.stream_calls.load(Ordering::SeqCst),
            live_provider.prompts().to_vec(),
            outcome.events,
            outcome.final_message,
            transcript,
        ),
        (
            2,
            vec!["".to_string(), "".to_string()],
            expected_events,
            expected_final_message,
            expected_transcript,
        ),
        "the live runtime should execute the tool, persist the result, and continue the assistant turn"
    );
}

#[derive(Debug, Clone)]
enum ScriptedResponse {
    ToolCall(ToolCall),
    Text(String),
}

struct ScriptedProvider {
    responses: Vec<ScriptedResponse>,
    prompts: Vec<String>,
    stream_calls: AtomicUsize,
}

impl ScriptedProvider {
    fn new(responses: Vec<ScriptedResponse>) -> Self {
        Self {
            responses,
            prompts: Vec::new(),
            stream_calls: AtomicUsize::new(0),
        }
    }

    fn prompts(&self) -> &[String] {
        &self.prompts
    }
}

#[async_trait]
impl Provider for ScriptedProvider {
    async fn complete(
        &mut self,
        _request: &RunRequest,
    ) -> Result<ProviderResponse, ProviderError> {
        Ok(ProviderResponse::text(
            self.responses
                .first()
                .map(|r| match r {
                    ScriptedResponse::Text(t) => t.clone(),
                    ScriptedResponse::ToolCall(_) => String::new(),
                })
                .unwrap_or_default(),
        ))
    }

    async fn stream(
        &mut self,
        request: &RunRequest,
        on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<ProviderResponse, ProviderError> {
        self.prompts.push(request.prompt.clone());
        let idx = self.stream_calls.fetch_add(1, Ordering::SeqCst);

        let response = self.responses.get(idx);

        match response {
            Some(ScriptedResponse::ToolCall(call)) => {
                let label = format!("call:{}({})", call.name, call.arguments);
                on_event(AgentEvent::MessageDelta(label.clone()));
                Ok(ProviderResponse::tool_calls(vec![call.clone()]))
            }
            Some(ScriptedResponse::Text(text)) => {
                for line in text.lines() {
                    on_event(AgentEvent::MessageDelta(line.to_string()));
                }
                Ok(ProviderResponse::text(text.clone()))
            }
            None => {
                Ok(ProviderResponse::text(String::new()))
            }
        }
    }
}

struct AddTool;

#[async_trait]
impl ToolExecutor for AddTool {
    fn descriptor(&self) -> &ToolDescriptor {
        static DESC: std::sync::OnceLock<ToolDescriptor> = std::sync::OnceLock::new();
        DESC.get_or_init(|| ToolDescriptor::new("add", "adds two numbers"))
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        assert_eq!(call.name, "add");
        ToolResult::success(&call, "5")
    }
}

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = std::env::temp_dir().join(format!(
        "matrixclaw-live-tool-execution-home-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&home).expect("create temp home");
    home
}
