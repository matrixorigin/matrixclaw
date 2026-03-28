use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_app_host::http::agent_api::AGENT_RUN_ROUTE;
use matrixclaw_app_host::http::SetupSurface;
use matrixclaw_app_host::live_runtime::session_db_path;
use matrixclaw_app_host::server::spawn_test_server;
use matrixclaw_app_host::ui_assets::UiAssetLayout;
use matrixclaw_session_runtime::message_projection::DurableTranscriptKind;
use matrixclaw_session_runtime::sqlite::SqliteStorage;
use matrixclaw_session_runtime::storage::TranscriptStore;
use serde_json::{json, Value};

#[test]
fn execution_node_smoke_harness() {
    let _env_lock = env_lock().lock().expect("env lock");
    let home = temp_home();
    let request_count = Arc::new(AtomicUsize::new(0));
    let provider_url = spawn_fixture_provider(request_count.clone());

    env::set_var("OPENROUTER_API_KEY", "test-key");
    env::set_var("MATRIXCLAW_OPENAI_BASE_URL", &provider_url);
    env::set_var("MATRIXCLAW_LLM_MODEL", "moonshotai/kimi-k2.5");

    let surface = SetupSurface::new(&home, UiAssetLayout::discover());
    let test_server = spawn_test_server(surface).expect("spawn test server");
    let client = reqwest::blocking::Client::new();
    let session_id = "execution-node-smoke-session";

    let response = client
        .post(format!(
            "http://{}/{}",
            test_server.address,
            AGENT_RUN_ROUTE.trim_start_matches('/')
        ))
        .header("content-type", "application/json")
        .body(
            json!({
                "prompt": "exercise the execution node through the host boundary",
                "session_id": session_id,
            })
            .to_string(),
        )
        .send()
        .expect("send execution node smoke request");

    assert_eq!(
        response.status(),
        200,
        "the host should still serve the execution node through the app-host boundary"
    );

    let body: Value = response.json().expect("execution node smoke response JSON");
    assert_eq!(
        body.get("session_id").and_then(Value::as_str),
        Some(session_id),
        "the smoke path should preserve the caller-provided session id"
    );
    assert_eq!(
        body.get("final_message").and_then(Value::as_str),
        Some("node-visible-result"),
        "the execution node smoke should surface the final runtime result"
    );

    let session_path = session_db_path(&home, session_id);
    let storage = SqliteStorage::open(&session_path).expect("open persisted session");
    let transcript = storage.load_transcript().expect("load transcript");
    assert!(
        transcript.iter().any(|entry| {
            entry.kind == DurableTranscriptKind::ToolResult
                && entry.content.contains("\"kind\":\"execution-node.capability-result\"")
                && entry.content.contains("\"backend\":\"node\"")
                && entry.content.contains("\"stdout\":\"node-visible-result\"")
        }),
        "the smoke should prove the execution node boundary is represented in persisted runtime state"
    );

    assert_eq!(
        request_count.load(Ordering::SeqCst),
        2,
        "the smoke should still route through the normal provider turn and tool continuation"
    );

    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonical repo root");
    let verifier = repo_root.join("scripts/verify-execution-node.sh");
    let verifier_text = fs::read_to_string(&verifier).expect("read execution node verifier");

    assert!(
        verifier_text.contains(
            "cargo test -p matrixclaw-app-host --test execution_node_contract execution_node_contract -- --exact"
        ),
        "the execution node harness should include the focused contract check"
    );
    assert!(
        verifier_text.contains(
            "cargo test -p matrixclaw-app-host --test execution_node_routing execution_node_routing -- --exact"
        ),
        "the execution node harness should include the routing check"
    );
    assert!(
        verifier_text.contains(
            "cargo test -p matrixclaw-app-host --test runtime_execution_node_integration runtime_execution_node_integration -- --exact"
        ),
        "the execution node harness should include the runtime integration check"
    );
    assert!(
        verifier_text.contains(
            "cargo test -p matrixclaw-app-host --test execution_node_smoke_harness execution_node_smoke_harness -- --exact"
        ),
        "the execution node harness should include the maintainer-facing smoke target"
    );

    test_server.shutdown().expect("shutdown test server");

    env::remove_var("OPENROUTER_API_KEY");
    env::remove_var("MATRIXCLAW_OPENAI_BASE_URL");
    env::remove_var("MATRIXCLAW_LLM_MODEL");
}

fn spawn_fixture_provider(request_count: Arc<AtomicUsize>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture provider");
    let address = listener.local_addr().expect("fixture provider address");

    thread::spawn(move || {
        for response in [
            fixture_stream_response("call:host.command(echo,node-visible-result)"),
            fixture_stream_response("node-visible-result"),
        ] {
            let (mut stream, _) = listener.accept().expect("accept fixture request");
            let _ = read_http_request(&mut stream);
            request_count.fetch_add(1, Ordering::SeqCst);
            stream
                .write_all(response.as_bytes())
                .expect("write fixture response");
        }
    });

    format!("http://{}", address)
}

fn fixture_stream_response(content: &str) -> String {
    format!(
        concat!(
            "HTTP/1.1 200 OK\r\n",
            "Content-Type: text/event-stream\r\n",
            "Connection: close\r\n",
            "\r\n",
            "data: {{\"choices\":[{{\"delta\":{{\"content\":\"{content}\"}}}}]}}\n\n",
            "data: {{\"choices\":[{{\"delta\":{{}},\"finish_reason\":\"stop\"}}]}}\n\n",
            "data: [DONE]\n\n"
        ),
        content = content
    )
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
        "matrixclaw-execution-node-smoke-home-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&home).expect("create temp home");
    home
}

fn env_lock() -> &'static std::sync::Mutex<()> {
    static LOCK: OnceLock<std::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
}
