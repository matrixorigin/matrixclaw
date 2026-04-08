use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use zstar_agent_core::event::AgentEvent;
use zstar_agent_core::provider::{Provider, ProviderError, ProviderResponse};
use zstar_agent_core::{RunMessageRole, RunRequest};
use zstar_app_host::gateway::matrix::{normalize_matrix_inbound_event, MatrixInboundEvent};
use zstar_app_host::gateway::store::GatewaySessionStore;
use zstar_app_host::ingress::run_ingress_with_provider;
use zstar_app_host::live_runtime::session_db_path;
use zstar_session_runtime::recovery::SessionRecoveryStore;
use zstar_session_runtime::sqlite::SqliteStorage;
use zstar_session_runtime::RuntimeMessage;

#[tokio::test]
async fn matrix_ingress_normalization() {
    let home = temp_home();
    {
        let _env_lock = env_lock().lock().expect("env lock");
    }
    let session_id = "matrix-session-1";
    seed_persisted_session(&home, session_id);

    let mut store = GatewaySessionStore::load_or_default(&home).expect("load gateway store");
    store
        .bind_matrix_thread("!room:example.org", Some("$thread"), session_id)
        .expect("bind room/thread to session");
    store.save(&home).expect("save gateway store");

    let event = MatrixInboundEvent {
        sender_id: "@alice:example.org".to_string(),
        sender_display_name: Some("Alice".to_string()),
        room_id: "!room:example.org".to_string(),
        thread_id: Some("$thread".to_string()),
        event_id: Some("$event".to_string()),
        target_agent: Some("planner".to_string()),
        body: "continue from Matrix".to_string(),
    };

    let envelope = normalize_matrix_inbound_event(&home, &event).expect("normalize Matrix event");
    assert_eq!(envelope.transport.kind, "matrix");
    assert_eq!(envelope.sender.id.as_deref(), Some("@alice:example.org"));
    assert_eq!(envelope.conversation.session_id, session_id);
    assert_eq!(envelope.conversation.thread_id.as_deref(), Some("$thread"));
    assert_eq!(envelope.target_agent.as_deref(), Some("planner"));
    assert_eq!(
        envelope.reply.channel_id.as_deref(),
        Some("!room:example.org")
    );
    assert_eq!(envelope.reply.reply_to.as_deref(), Some("$event"));

    let mut provider = RecordingProvider::new("matrix resumed");
    let outcome =
        run_ingress_with_provider(&home, "moonshotai/kimi-k2.5", &envelope, &mut provider)
            .await
            .expect("resume mapped Matrix session through shared runtime");

    assert_eq!(outcome.live_run.session_id, session_id);
    assert!(
        provider.context_messages[0]
            .iter()
            .any(|message| message == "assistant:browser seeded reply"),
        "persisted assistant history should reach the resumed runtime turn"
    );
    assert!(
        provider.context_messages[0]
            .iter()
            .any(|message| message == "user:continue from Matrix"),
        "new Matrix prompt should reach the resumed runtime turn"
    );

    let storage = SqliteStorage::open(session_db_path(&home, session_id)).expect("open session db");
    let snapshot = storage
        .load_recovery_snapshot()
        .expect("load session snapshot");
    assert!(snapshot
        .history
        .contains(&RuntimeMessage::User("continue from Matrix".to_string())));
    assert!(snapshot
        .history
        .contains(&RuntimeMessage::Assistant("matrix resumed".to_string())));
}

struct RecordingProvider {
    context_messages: Vec<Vec<String>>,
    response: String,
}

impl RecordingProvider {
    fn new(response: &str) -> Self {
        Self {
            context_messages: Vec::new(),
            response: response.to_string(),
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

fn seed_persisted_session(home: &PathBuf, session_id: &str) {
    let path = session_db_path(home, session_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create session parent");
    }
    let mut storage = SqliteStorage::open(path).expect("open session storage");
    storage
        .persist_session(&zstar_session_runtime::session::Session::new(vec![
            RuntimeMessage::Assistant("browser seeded reply".to_string()),
        ]))
        .expect("persist seeded session");
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
        "zstar-matrix-ingress-home-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&home).expect("create temp home");
    home
}
