use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_agent_core::event::AgentEvent;
use matrixclaw_agent_core::provider::{Provider, ProviderError};
use matrixclaw_agent_core::RunRequest;
use matrixclaw_app_host::gateway::client::GatewayTransportClient;
use matrixclaw_app_host::gateway::matrix::MatrixInboundEvent;
use matrixclaw_app_host::gateway::runtime::{GatewayRunStatus, GatewayRuntime};
use matrixclaw_app_host::gateway::store::GatewaySessionStore;
use matrixclaw_app_host::gateway::transport::{MatrixGatewayTransport, MatrixTransportConfig};
use matrixclaw_app_host::gateway::{GatewayOutboundDelivery, OutboundDeliveryKind};

#[test]
fn matrix_gateway_transport_runner_streams_deliveries() {
    let _env_lock = env_lock().lock().expect("env lock");
    let home = temp_home();

    let mut store = GatewaySessionStore::load_or_default(&home).expect("load gateway store");
    store
        .bind_matrix_thread("!room:example.org", Some("$thread"), "matrix-session")
        .expect("bind room thread");
    store.save(&home).expect("save gateway store");

    let inbound = MatrixInboundEvent {
        sender_id: "@alice:example.org".to_string(),
        sender_display_name: Some("Alice".to_string()),
        room_id: "!room:example.org".to_string(),
        thread_id: Some("$thread".to_string()),
        event_id: Some("$event".to_string()),
        target_agent: Some("planner".to_string()),
        body: "stream this reply".to_string(),
    };
    let mut client = FakeMatrixClient::new(vec![inbound]);
    let mut provider = StreamingProvider::new(vec!["first ".to_string(), "second".to_string()]);
    let transport = MatrixGatewayTransport::new(
        &home,
        MatrixTransportConfig {
            model: "moonshotai/kimi-k2.5".to_string(),
        },
    );

    let report = transport
        .run_once(&mut client, &mut provider)
        .expect("run matrix gateway transport")
        .expect("expected one inbound event");

    assert_eq!(report.status, GatewayRunStatus::Processed);
    assert_eq!(report.session_id.as_deref(), Some("matrix-session"));
    assert_eq!(report.deliveries_sent, 3);
    assert_eq!(report.retries_recorded, 0);
    assert_eq!(
        client
            .sent
            .iter()
            .map(|delivery| delivery.kind.clone())
            .collect::<Vec<_>>(),
        vec![
            OutboundDeliveryKind::AssistantChunk,
            OutboundDeliveryKind::AssistantChunk,
            OutboundDeliveryKind::AssistantFinal,
        ]
    );
    assert_eq!(client.sent[0].body, "first ");
    assert_eq!(client.sent[1].body, "second");
    assert_eq!(client.sent[2].body, "first second");
}

#[test]
fn matrix_gateway_transport_runner_records_and_flushes_retries() {
    let _env_lock = env_lock().lock().expect("env lock");
    let home = temp_home();

    let inbound = MatrixInboundEvent {
        sender_id: "@alice:example.org".to_string(),
        sender_display_name: Some("Alice".to_string()),
        room_id: "!retry-room:example.org".to_string(),
        thread_id: Some("$thread".to_string()),
        event_id: Some("$event".to_string()),
        target_agent: None,
        body: "retry this send".to_string(),
    };
    let mut failing_client = FakeMatrixClient::new(vec![inbound]);
    failing_client.failures_remaining = 1;

    let mut provider = StreamingProvider::new(vec!["retry body".to_string()]);
    let transport = MatrixGatewayTransport::new(
        &home,
        MatrixTransportConfig {
            model: "moonshotai/kimi-k2.5".to_string(),
        },
    );

    let report = transport
        .run_once(&mut failing_client, &mut provider)
        .expect("run matrix gateway transport")
        .expect("expected one inbound event");

    assert_eq!(report.status, GatewayRunStatus::Processed);
    assert_eq!(report.deliveries_sent, 1);
    assert_eq!(report.retries_recorded, 1);
    let runtime = GatewayRuntime::load_or_default(&home).expect("reload gateway runtime");
    assert_eq!(runtime.pending_retries().len(), 1);
    assert_eq!(
        runtime.pending_retries()[0].kind,
        OutboundDeliveryKind::AssistantChunk
    );
    assert_eq!(runtime.pending_retries()[0].body, "retry body");

    let mut recovery_client = FakeMatrixClient::default();
    let flushed = transport
        .flush_pending_retries(&mut recovery_client)
        .expect("flush pending retries");
    assert_eq!(flushed, 1);
    assert_eq!(recovery_client.sent.len(), 1);
    assert_eq!(
        recovery_client.sent[0].kind,
        OutboundDeliveryKind::AssistantChunk
    );
    assert_eq!(recovery_client.sent[0].body, "retry body");
    let reloaded = GatewayRuntime::load_or_default(&home).expect("reload gateway runtime");
    assert!(reloaded.pending_retries().is_empty());
}

#[derive(Debug, Default)]
struct FakeMatrixClient {
    inbound: Vec<MatrixInboundEvent>,
    sent: Vec<GatewayOutboundDelivery>,
    failures_remaining: usize,
}

impl FakeMatrixClient {
    fn new(inbound: Vec<MatrixInboundEvent>) -> Self {
        Self {
            inbound,
            sent: Vec::new(),
            failures_remaining: 0,
        }
    }
}

impl GatewayTransportClient<MatrixInboundEvent, GatewayOutboundDelivery> for FakeMatrixClient {
    fn recv_inbound(&mut self) -> Result<Option<MatrixInboundEvent>, String> {
        if self.inbound.is_empty() {
            return Ok(None);
        }
        Ok(Some(self.inbound.remove(0)))
    }

    fn send_delivery(&mut self, delivery: GatewayOutboundDelivery) -> Result<(), String> {
        if self.failures_remaining > 0 {
            self.failures_remaining -= 1;
            return Err("transient Matrix send failure".to_string());
        }

        self.sent.push(delivery);
        Ok(())
    }
}

struct StreamingProvider {
    chunks: Vec<String>,
}

impl StreamingProvider {
    fn new(chunks: Vec<String>) -> Self {
        Self { chunks }
    }
}

impl Provider for StreamingProvider {
    fn complete(&mut self, _request: &RunRequest) -> Result<String, ProviderError> {
        Err(ProviderError(
            "streaming provider only supports streamed live runs".to_string(),
        ))
    }

    fn stream(
        &mut self,
        _request: &RunRequest,
        on_event: &mut dyn FnMut(AgentEvent),
    ) -> Result<String, ProviderError> {
        on_event(AgentEvent::RunStarted);
        on_event(AgentEvent::MessageStarted);
        let mut final_message = String::new();
        for chunk in &self.chunks {
            final_message.push_str(chunk);
            on_event(AgentEvent::MessageDelta(chunk.clone()));
        }
        on_event(AgentEvent::MessageCompleted(final_message.clone()));
        Ok(final_message)
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
        "matrixclaw-matrix-transport-home-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&home).expect("create temp home");
    home
}
