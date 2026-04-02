use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use matrixclaw_agent_core::event::AgentEvent;
use matrixclaw_agent_core::provider::{Provider, ProviderError, ProviderResponse};
use matrixclaw_agent_core::{RunMessageRole, RunRequest};
use matrixclaw_app_host::gateway::matrix::MatrixInboundEvent;
use matrixclaw_app_host::gateway::runtime::GatewayRuntime;
use matrixclaw_app_host::gateway::store::GatewaySessionStore;
use matrixclaw_app_host::http::agent_api::AGENT_RUN_ROUTE;
use matrixclaw_app_host::http::{HttpRequest, SetupSurface};
use matrixclaw_app_host::ui_assets::UiAssetLayout;
use serde_json::json;

#[test]
fn browser_matrix_session_reuse() {
    let _env_lock = env_lock().lock().expect("env lock");
    let home = temp_home();
    let request_count = Arc::new(AtomicUsize::new(0));
    let request_bodies = Arc::new(Mutex::new(Vec::new()));
    let server_url = spawn_fixture_server(request_count.clone(), request_bodies.clone());

    env::set_var("OPENROUTER_API_KEY", "test-key");
    env::set_var("MATRIXCLAW_OPENAI_BASE_URL", &server_url);
    env::set_var("MATRIXCLAW_LLM_MODEL", "moonshotai/kimi-k2.5");

    let surface = SetupSurface::new(&home, UiAssetLayout::discover());
    let session_id = "browser-matrix-session";

    let browser_response = surface.handle(HttpRequest::post(
        AGENT_RUN_ROUTE,
        json!({
            "prompt": "seed the browser transport",
            "session_id": session_id,
        })
        .to_string(),
    ));
    assert_eq!(
        browser_response.status_code, 200,
        "browser run should succeed"
    );

    let steering_response = surface.handle(HttpRequest::post(
        "/api/queue/steering",
        json!({
            "kind": "steering",
            "message": "apply steering across transports",
            "session_id": session_id,
        })
        .to_string(),
    ));
    assert_eq!(steering_response.status_code, 200);

    let follow_up_response = surface.handle(HttpRequest::post(
        "/api/queue/follow-up",
        json!({
            "kind": "follow-up",
            "message": "defer this follow-up",
            "session_id": session_id,
        })
        .to_string(),
    ));
    assert_eq!(follow_up_response.status_code, 200);

    let mut store = GatewaySessionStore::load_or_default(&home).expect("load gateway store");
    store
        .bind_matrix_thread("!room:example.org", Some("$thread"), session_id)
        .expect("bind Matrix thread");
    store.save(&home).expect("save gateway store");

    let event = MatrixInboundEvent {
        sender_id: "@alice:example.org".to_string(),
        sender_display_name: Some("Alice".to_string()),
        room_id: "!room:example.org".to_string(),
        thread_id: Some("$thread".to_string()),
        event_id: Some("$event".to_string()),
        target_agent: Some("planner".to_string()),
        body: "resume from Matrix".to_string(),
    };

    let mut runtime = GatewayRuntime::load_or_default(&home).expect("load gateway runtime");
    let mut provider = RecordingProvider::new("matrix reply");

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime");
    let outcome = rt
        .block_on(runtime.process_matrix_event(
            &home,
            "moonshotai/kimi-k2.5",
            &event,
            &mut provider,
        ))
        .expect("resume mapped Matrix event");

    env::remove_var("OPENROUTER_API_KEY");
    env::remove_var("MATRIXCLAW_OPENAI_BASE_URL");
    env::remove_var("MATRIXCLAW_LLM_MODEL");

    assert_eq!(
        outcome.session_id.as_deref(),
        Some(session_id),
        "Matrix gateway should reuse the browser-created session"
    );
    assert_eq!(
        request_count.load(Ordering::SeqCst),
        1,
        "only the browser transport should use the fixture HTTP provider in this smoke"
    );
    assert!(
        provider.context_messages[0]
            .iter()
            .any(|message| message == "assistant:browser seeded reply"),
        "Matrix resume should see browser assistant history"
    );
    assert!(
        provider.context_messages[0]
            .iter()
            .any(|message| message == "user:resume from Matrix"),
        "Matrix user message should reach the shared runtime"
    );
    assert!(
        provider.context_messages[0]
            .iter()
            .any(|message| message == "system:apply steering across transports"),
        "steering queued through the browser boundary should reach the next Matrix turn"
    );
    assert!(
        !provider.context_messages[0]
            .iter()
            .any(|message| message == "user:defer this follow-up"),
        "follow-up should stay deferred for the next run"
    );

    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonical repo root");
    assert!(
        repo_root.join("scripts/verify-matrix-gateway.sh").exists(),
        "the Matrix smoke should ship with a maintainer-facing harness"
    );
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

fn render_run_message(message: &matrixclaw_agent_core::RunMessage) -> String {
    let role = match message.role {
        RunMessageRole::User => "user",
        RunMessageRole::System => "system",
        RunMessageRole::Assistant => "assistant",
        RunMessageRole::Tool => "tool",
    };
    format!("{role}:{}", message.content)
}

fn spawn_fixture_server(
    request_count: Arc<AtomicUsize>,
    request_bodies: Arc<Mutex<Vec<String>>>,
) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture server");
    let address = listener.local_addr().expect("fixture server addr");

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept fixture request");
        let request = read_http_request(&mut stream);
        request_count.fetch_add(1, Ordering::SeqCst);
        request_bodies
            .lock()
            .expect("lock request body sink")
            .push(request);

        let response = concat!(
            "HTTP/1.1 200 OK\r\n",
            "Content-Type: text/event-stream\r\n",
            "Connection: close\r\n",
            "\r\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"browser seeded reply\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
            "data: [DONE]\n\n"
        );

        stream
            .write_all(response.as_bytes())
            .expect("write fixture response");
    });

    format!("http://{address}")
}

fn read_http_request(stream: &mut std::net::TcpStream) -> String {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 4096];
    let mut header_end = None;
    let mut content_length = 0_usize;

    loop {
        let read = stream.read(&mut chunk).expect("read fixture request");
        assert!(read > 0, "fixture request closed before body arrived");
        buffer.extend_from_slice(&chunk[..read]);

        if header_end.is_none() {
            header_end = find_header_end(&buffer);
            if let Some(end) = header_end {
                content_length = parse_content_length(&buffer[..end]);
            }
        }

        if let Some(end) = header_end {
            let body_len = buffer.len().saturating_sub(end);
            if body_len >= content_length {
                break;
            }
        }
    }

    String::from_utf8(buffer).expect("fixture request should be utf-8")
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
}

fn parse_content_length(headers: &[u8]) -> usize {
    String::from_utf8_lossy(headers)
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("content-length") {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0)
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!(
        "matrixclaw-browser-matrix-home-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&home).expect("create temp home");
    home
}
