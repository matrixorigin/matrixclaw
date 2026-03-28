use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_app_host::http::agent_api::AGENT_RUN_ROUTE;
use matrixclaw_app_host::http::{HttpRequest, SetupSurface};
use matrixclaw_app_host::live_runtime::session_db_path;
use matrixclaw_app_host::paths;
use matrixclaw_app_host::ui_assets::UiAssetLayout;
use matrixclaw_session_runtime::message_projection::DurableTranscriptKind;
use matrixclaw_session_runtime::sqlite::SqliteStorage;
use matrixclaw_session_runtime::storage::TranscriptStore;
use serde_json::{json, Value};

#[test]
fn session_backed_live_run_service() {
    let _env_lock = env_lock().lock().expect("env lock");
    let home = temp_home();
    let request_count = Arc::new(AtomicUsize::new(0));
    let server_url = spawn_fixture_server(request_count.clone());

    env::set_var("OPENROUTER_API_KEY", "test-key");
    env::set_var("MATRIXCLAW_OPENAI_BASE_URL", &server_url);
    env::set_var("MATRIXCLAW_LLM_MODEL", "moonshotai/kimi-k2.5");

    let surface = SetupSurface::new(&home, UiAssetLayout::discover());
    let response = surface.handle(HttpRequest::post(
        AGENT_RUN_ROUTE,
        json!({ "prompt": "Say persisted hello" }).to_string(),
    ));

    env::remove_var("OPENROUTER_API_KEY");
    env::remove_var("MATRIXCLAW_OPENAI_BASE_URL");
    env::remove_var("MATRIXCLAW_LLM_MODEL");

    assert_eq!(response.status_code, 200, "live run should succeed");
    assert_eq!(
        request_count.load(Ordering::SeqCst),
        1,
        "live service should still use a single provider turn"
    );

    let body: Value =
        serde_json::from_slice(&response.body).expect("agent run response should be valid JSON");

    assert_eq!(
        body.get("final_message").and_then(Value::as_str),
        Some("Persisted hello"),
        "assistant output should be returned at the service boundary"
    );

    let session_id = body
        .get("session_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .expect("first live prompt should create a persisted session id");

    let events = body
        .get("events")
        .and_then(Value::as_array)
        .expect("service response should expose ordered runtime events for the browser transcript");
    assert!(
        events.iter().any(|event| {
            event.get("kind").and_then(Value::as_str) == Some("message_completed")
                && event.get("content").and_then(Value::as_str) == Some("Persisted hello")
        }),
        "visible assistant completion should be present in the runtime event stream"
    );

    let session_path = session_db_path(&home, session_id);
    assert!(
        session_path.exists(),
        "live runtime should persist session state at {:?}",
        session_path
    );

    let storage = SqliteStorage::open(&session_path).expect("open persisted session");
    let transcript = storage
        .load_transcript()
        .expect("load persisted transcript");
    assert_eq!(
        transcript.len(),
        1,
        "assistant completion should persist visibly once"
    );
    assert_eq!(transcript[0].kind, DurableTranscriptKind::Assistant);
    assert_eq!(transcript[0].content, "Persisted hello");
}

#[test]
fn session_db_path_stays_inside_runtime_home() {
    let home = temp_home();
    let path = session_db_path(&home, "../../outside");
    let sessions_root = paths::runtime_home(&home).join("state").join("sessions");

    assert!(
        path.starts_with(&sessions_root),
        "session paths must stay inside {:?}, got {:?}",
        sessions_root,
        path
    );
    assert!(
        !path.to_string_lossy().contains("../"),
        "session paths must not preserve traversal segments: {:?}",
        path
    );
}

fn spawn_fixture_server(request_count: Arc<AtomicUsize>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture server");
    let address = listener.local_addr().expect("fixture server addr");

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept fixture request");
        let _request = read_http_request(&mut stream);
        request_count.fetch_add(1, Ordering::SeqCst);

        let response = concat!(
            "HTTP/1.1 200 OK\r\n",
            "Content-Type: text/event-stream\r\n",
            "Connection: close\r\n",
            "\r\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"Persisted \"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
            "data: [DONE]\n\n"
        );

        stream
            .write_all(response.as_bytes())
            .expect("write fixture response");
    });

    format!("http://{}", address)
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
    let headers = String::from_utf8(headers.to_vec()).expect("headers should be utf-8");

    headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if !name.eq_ignore_ascii_case("content-length") {
                return None;
            }

            value.trim().parse::<usize>().ok()
        })
        .unwrap_or(0)
}

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!(
        "matrixclaw-session-runtime-home-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&home).expect("create temp home");
    home
}

fn env_lock() -> &'static Mutex<()> {
    static ENV_LOCK: Mutex<()> = Mutex::new(());
    &ENV_LOCK
}
