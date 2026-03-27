use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_agent_core::event::AgentEvent;
use matrixclaw_agent_core::provider::{Provider, ProviderError};
use matrixclaw_agent_core::{RunMessageRole, RunRequest};
use matrixclaw_app_host::live_runtime::{
    session_db_path, LiveRunRequest, SessionBackedLiveRunService,
};
use matrixclaw_compat_openclaw::http::openclaw_chat_http;
use matrixclaw_compat_openclaw::translation::{
    openclaw_session_db_path, OpenClawChatMessage, OpenClawChatRequest,
};
use matrixclaw_session_runtime::sqlite::SqliteStorage;
use matrixclaw_session_runtime::recovery::SessionRecoveryStore;
use matrixclaw_session_runtime::{ChatEvent, ChatRequest, ChatRuntime, RuntimeMessage};

#[test]
fn openclaw_seeded_session_is_reused_by_shared_live_runtime() {
    let _env_lock = env_lock().lock().expect("env lock");
    let home = temp_home();
    env::set_var("HOME", &home);

    let conversation_id = format!(
        "openclaw-shared-runtime-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos()
    );
    let request = OpenClawChatRequest::new(
        conversation_id.clone(),
        vec![
            OpenClawChatMessage::user("seed through the OpenClaw boundary"),
            OpenClawChatMessage::system("keep the compatibility layer protocol-shaped"),
        ],
    );

    let mut compat_runtime = RecordingRuntime::default();
    let response = openclaw_chat_http(&request, &mut compat_runtime);

    let compat_session_path = openclaw_session_db_path(&conversation_id);
    let live_session_path = session_db_path(&home, &conversation_id);

    assert_eq!(
        compat_session_path, live_session_path,
        "OpenClaw and app-host should share the same on-disk session layout"
    );
    assert!(
        compat_session_path.exists(),
        "compat chat should persist a session before the live runtime resumes it"
    );
    assert_eq!(response.conversation_id, conversation_id);
    assert_eq!(
        response.frames,
        vec![
            matrixclaw_compat_openclaw::stream_adapter::ChatFrame::AssistantChunk {
                content: "compatibility answer".to_string(),
            },
            matrixclaw_compat_openclaw::stream_adapter::ChatFrame::Completed,
        ],
        "the compatibility boundary should continue to project OpenClaw chat frames"
    );

    let service = SessionBackedLiveRunService::new(&home);
    let mut provider = RecordingProvider::default();
    let outcome = service
        .run_with_provider(
            "moonshotai/kimi-k2.5",
            LiveRunRequest {
                prompt: "resume the same conversation".to_string(),
                session_id: Some(conversation_id.clone()),
            },
            &mut provider,
        )
        .expect("live runtime should resume the OpenClaw-seeded session");

    assert_eq!(
        outcome.session_id, conversation_id,
        "the shared runtime should continue the same persisted session"
    );
    assert_eq!(provider.prompts.len(), 1, "the shared runtime should make one provider turn");
    assert_eq!(
        provider.context_messages.len(),
        1,
        "the resumed run should expose one projected context payload"
    );
    assert!(
        provider.context_messages[0]
            .iter()
            .any(|message| message == "assistant:compatibility answer"),
        "the live runtime should see the assistant history seeded by OpenClaw persistence"
    );
    assert!(
        provider.context_messages[0]
            .iter()
            .any(|message| message == "user:resume the same conversation"),
        "the live runtime should still include the new browser prompt"
    );

    let storage = SqliteStorage::open(&live_session_path).expect("open shared session storage");
    let snapshot = storage
        .load_recovery_snapshot()
        .expect("load shared session snapshot");
    assert!(
        snapshot
            .history
            .contains(&RuntimeMessage::Assistant("compatibility answer".to_string())),
        "the OpenClaw-seeded assistant turn should survive in the shared session history"
    );
    assert!(
        snapshot
            .history
            .contains(&RuntimeMessage::User("resume the same conversation".to_string())),
        "the live runtime should persist the resumed browser prompt into the same session"
    );
    assert!(
        snapshot
            .history
            .contains(&RuntimeMessage::Assistant("Persisted hello".to_string())),
        "the live runtime should append its own assistant completion to the shared session"
    );

    env::remove_var("HOME");
}

#[derive(Default)]
struct RecordingRuntime {
    seen_requests: Vec<ChatRequest>,
}

impl ChatRuntime for RecordingRuntime {
    fn handle_chat(&mut self, request: ChatRequest) -> Vec<ChatEvent> {
        self.seen_requests.push(request);
        vec![
            ChatEvent::AssistantChunk("compatibility answer".to_string()),
            ChatEvent::Completed,
        ]
    }
}

#[derive(Default)]
struct RecordingProvider {
    prompts: Vec<String>,
    context_messages: Vec<Vec<String>>,
}

impl Provider for RecordingProvider {
    fn complete(&mut self, _request: &RunRequest) -> Result<String, ProviderError> {
        Err(ProviderError(
            "recording provider only supports streamed live runs".to_string(),
        ))
    }

    fn stream(
        &mut self,
        request: &RunRequest,
        on_event: &mut dyn FnMut(AgentEvent),
    ) -> Result<String, ProviderError> {
        self.prompts.push(request.prompt.clone());
        self.context_messages.push(
            request
                .context_messages
                .iter()
                .map(render_run_message)
                .collect(),
        );

        on_event(AgentEvent::RunStarted);
        on_event(AgentEvent::MessageStarted);
        on_event(AgentEvent::MessageDelta("Persisted ".to_string()));
        on_event(AgentEvent::MessageDelta("hello".to_string()));
        on_event(AgentEvent::MessageCompleted("Persisted hello".to_string()));

        Ok("Persisted hello".to_string())
    }
}

fn render_run_message(message: &matrixclaw_agent_core::RunMessage) -> String {
    let role = match message.role {
        RunMessageRole::User => "user",
        RunMessageRole::System => "system",
        RunMessageRole::Assistant => "assistant",
        RunMessageRole::Tool => "tool",
    };

    format!("{role}:{}", message.content)
}

fn env_lock() -> &'static Mutex<()> {
    static ENV_LOCK: Mutex<()> = Mutex::new(());
    &ENV_LOCK
}

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!(
        "matrixclaw-openclaw-shared-home-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&home).expect("create temp home");
    home
}
