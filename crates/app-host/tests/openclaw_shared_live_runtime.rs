use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use zstar_agent_core::event::AgentEvent;
use zstar_agent_core::provider::{Provider, ProviderError, ProviderResponse};
use zstar_agent_core::{RunMessageRole, RunRequest};
use zstar_app_host::live_runtime::{session_db_path, LiveRunRequest, SessionBackedLiveRunService};
use zstar_app_host::openclaw_transport::openclaw_chat_http_with_provider;
use zstar_compat_openclaw::translation::{OpenClawChatMessage, OpenClawChatRequest};
use zstar_session_runtime::recovery::SessionRecoveryStore;
use zstar_session_runtime::sqlite::SqliteStorage;
use zstar_session_runtime::RuntimeMessage;

#[tokio::test]
async fn openclaw_transport_reuses_the_shared_live_runtime() {
    let home = temp_home();
    {
        let _env_lock = env_lock().lock().expect("env lock");
        env::set_var("HOME", &home);
    }

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
            OpenClawChatMessage::system("keep the compatibility layer protocol-shaped"),
            OpenClawChatMessage::user("seed through the OpenClaw boundary"),
        ],
    );

    let mut compat_provider = RecordingProvider::compat();
    let response = openclaw_chat_http_with_provider(
        &home,
        "moonshotai/kimi-k2.5",
        &request,
        &mut compat_provider,
    )
    .await
    .expect("serve OpenClaw request through the shared runtime");

    let live_session_path = session_db_path(&home, &conversation_id);

    assert!(
        live_session_path.exists(),
        "OpenClaw transport should persist a session before the live runtime resumes it"
    );
    assert_eq!(response.conversation_id, conversation_id);
    assert_eq!(
        response.frames,
        vec![
            zstar_compat_openclaw::stream_adapter::ChatFrame::AssistantChunk {
                content: "compatibility answer".to_string(),
            },
            zstar_compat_openclaw::stream_adapter::ChatFrame::Completed,
        ],
        "the app-host transport should continue to project OpenClaw chat frames"
    );
    assert_eq!(
        compat_provider.context_messages.len(),
        1,
        "OpenClaw transport should execute through the live runtime provider path"
    );
    assert!(
        compat_provider.context_messages[0]
            .iter()
            .any(|message| message == "system:keep the compatibility layer protocol-shaped"),
        "OpenClaw transport should seed protocol context into the shared runtime history"
    );
    assert!(
        compat_provider.context_messages[0]
            .iter()
            .any(|message| message == "user:seed through the OpenClaw boundary"),
        "OpenClaw transport should send the current protocol user turn through the live runtime"
    );

    let service = SessionBackedLiveRunService::new(&home).await;
    let mut provider = RecordingProvider::live();
    let outcome = service
        .run_with_provider(
            "moonshotai/kimi-k2.5",
            LiveRunRequest {
                prompt: "resume the same conversation".to_string(),
                session_id: Some(conversation_id.clone()),
            },
            &mut provider,
        )
        .await
        .expect("live runtime should resume the OpenClaw-seeded session");

    assert_eq!(
        outcome.session_id, conversation_id,
        "the shared runtime should continue the same persisted session"
    );
    assert_eq!(
        provider.prompts.len(),
        1,
        "the shared runtime should make one provider turn"
    );
    assert_eq!(
        provider.context_messages.len(),
        1,
        "the resumed run should expose one projected context payload"
    );
    assert!(
        provider.context_messages[0]
            .iter()
            .any(|message| message == "assistant:compatibility answer"),
        "the live runtime should see the assistant history seeded by the OpenClaw transport"
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
        snapshot.history.contains(&RuntimeMessage::RuntimeSummary(
            "keep the compatibility layer protocol-shaped".to_string()
        )),
        "the OpenClaw system context should survive in the shared session history"
    );
    assert!(
        snapshot.history.contains(&RuntimeMessage::User(
            "seed through the OpenClaw boundary".to_string()
        )),
        "the OpenClaw prompt should be persisted into the shared session history"
    );
    assert!(
        snapshot.history.contains(&RuntimeMessage::Assistant(
            "compatibility answer".to_string()
        )),
        "the OpenClaw-seeded assistant turn should survive in the shared session history"
    );
    assert!(
        snapshot.history.contains(&RuntimeMessage::User(
            "resume the same conversation".to_string()
        )),
        "the live runtime should persist the resumed browser prompt into the same session"
    );
    assert!(
        snapshot
            .history
            .contains(&RuntimeMessage::Assistant("Persisted hello".to_string())),
        "the live runtime should append its own assistant completion to the shared session"
    );

    {
        let _env_lock = env_lock().lock().expect("env lock");
        env::remove_var("HOME");
    }
}

struct RecordingProvider {
    prompts: Vec<String>,
    context_messages: Vec<Vec<String>>,
    response: String,
}

impl RecordingProvider {
    fn live() -> Self {
        Self {
            prompts: Vec::new(),
            context_messages: Vec::new(),
            response: "Persisted hello".to_string(),
        }
    }

    fn compat() -> Self {
        Self {
            prompts: Vec::new(),
            context_messages: Vec::new(),
            response: "compatibility answer".to_string(),
        }
    }
}

#[async_trait]
impl Provider for RecordingProvider {
    async fn complete(&mut self, _request: &RunRequest) -> Result<ProviderResponse, ProviderError> {
        Err(ProviderError(
            "recording provider only supports streamed live runs".to_string(),
        ))
    }

    async fn stream(
        &mut self,
        request: &RunRequest,
        on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<ProviderResponse, ProviderError> {
        self.prompts.push(request.prompt.clone());
        self.context_messages.push(
            request
                .context_messages
                .iter()
                .map(render_run_message)
                .collect(),
        );

        on_event(AgentEvent::MessageStarted);
        on_event(AgentEvent::MessageDelta(self.response.clone()));
        on_event(AgentEvent::MessageCompleted(self.response.clone()));

        Ok(ProviderResponse::text(self.response.clone()))
    }
}

fn render_run_message(message: &zstar_agent_core::RunMessage) -> String {
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
        "zstar-openclaw-shared-home-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&home).expect("create temp home");
    home
}
