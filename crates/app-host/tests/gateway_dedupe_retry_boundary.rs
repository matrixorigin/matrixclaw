use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use zstar_agent_core::event::AgentEvent;
use zstar_agent_core::provider::{Provider, ProviderError, ProviderResponse};
use zstar_agent_core::RunRequest;
use zstar_app_host::gateway::matrix::MatrixInboundEvent;
use zstar_app_host::gateway::runtime::{GatewayDeliveryRetry, GatewayRunStatus, GatewayRuntime};
use zstar_app_host::gateway::store::GatewaySessionStore;
use zstar_app_host::gateway::OutboundDeliveryKind;

#[tokio::test]
async fn gateway_dedupe_retry_boundary() {
    let home = temp_home();
    {
        let _env_lock = env_lock().lock().expect("env lock");
    }

    let mut store = GatewaySessionStore::load_or_default(&home).expect("load gateway store");
    store
        .bind_matrix_thread("!room:example.org", Some("$thread"), "matrix-session")
        .expect("bind thread");
    store.save(&home).expect("save gateway store");

    let event = MatrixInboundEvent {
        sender_id: "@alice:example.org".to_string(),
        sender_display_name: Some("Alice".to_string()),
        room_id: "!room:example.org".to_string(),
        thread_id: Some("$thread".to_string()),
        event_id: Some("$event".to_string()),
        target_agent: Some("planner".to_string()),
        body: "handle this once".to_string(),
    };

    let mut runtime = GatewayRuntime::load_or_default(&home).expect("load gateway runtime");
    let mut provider = CountingProvider::default();

    let first = runtime
        .process_matrix_event(&home, "moonshotai/kimi-k2.5", &event, &mut provider)
        .await
        .expect("process first event");
    assert_eq!(first.status, GatewayRunStatus::Processed);
    assert_eq!(provider.calls, 1, "first delivery should reach the runtime");

    let second = runtime
        .process_matrix_event(&home, "moonshotai/kimi-k2.5", &event, &mut provider)
        .await
        .expect("process duplicate event");
    assert_eq!(second.status, GatewayRunStatus::Duplicate);
    assert_eq!(
        provider.calls, 1,
        "duplicate gateway delivery should not create another runtime turn"
    );

    runtime
        .record_retry(GatewayDeliveryRetry {
            gateway_kind: "matrix".to_string(),
            kind: OutboundDeliveryKind::AssistantFinal,
            channel_id: "!room:example.org".to_string(),
            thread_id: Some("$thread".to_string()),
            reply_to: Some("$event".to_string()),
            body: "retry this send".to_string(),
        })
        .expect("record retry");

    runtime.save(&home).expect("save gateway runtime state");
    let reloaded = GatewayRuntime::load_or_default(&home).expect("reload runtime state");
    assert_eq!(reloaded.pending_retries().len(), 1);
    assert_eq!(reloaded.pending_retries()[0].body, "retry this send");
}

#[derive(Default)]
struct CountingProvider {
    calls: usize,
}

#[async_trait]
impl Provider for CountingProvider {
    async fn complete(&mut self, _request: &RunRequest) -> Result<ProviderResponse, ProviderError> {
        Err(ProviderError(
            "counting provider only supports streamed live runs".to_string(),
        ))
    }

    async fn stream(
        &mut self,
        _request: &RunRequest,
        on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<ProviderResponse, ProviderError> {
        self.calls += 1;
        on_event(AgentEvent::MessageStarted);
        on_event(AgentEvent::MessageDelta("gateway response".to_string()));
        on_event(AgentEvent::MessageCompleted("gateway response".to_string()));
        Ok(ProviderResponse::text("gateway response"))
    }
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
        "zstar-gateway-runtime-home-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&home).expect("create temp home");
    home
}
