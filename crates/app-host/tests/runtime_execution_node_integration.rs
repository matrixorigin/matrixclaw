use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use zstar_app_host::http::agent_api::AGENT_RUN_ROUTE;
use zstar_app_host::http::SetupSurface;
use zstar_app_host::live_runtime::session_db_path;
use zstar_app_host::server::spawn_test_server;
use zstar_app_host::ui_assets::UiAssetLayout;
use zstar_session_runtime::message_projection::DurableTranscriptKind;
use zstar_session_runtime::sqlite::SqliteStorage;
use zstar_session_runtime::storage::TranscriptStore;

#[tokio::test]
async fn runtime_execution_node_integration() {
    let home = temp_home();
    let request_count = Arc::new(AtomicUsize::new(0));
    let provider_url = spawn_fixture_provider(request_count.clone());

    {
        let _env_lock = env_lock().lock().expect("env lock");
        env::set_var("OPENROUTER_API_KEY", "test-key");
        env::set_var("MATRIXCLAW_OPENAI_BASE_URL", &provider_url);
        env::set_var("MATRIXCLAW_LLM_MODEL", "moonshotai/kimi-k2.5");
    }

    let surface = SetupSurface::new(&home, UiAssetLayout::discover());
    let test_server = spawn_test_server(surface).expect("spawn test server");

    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "http://{}/{}",
            test_server.address,
            AGENT_RUN_ROUTE.trim_start_matches('/')
        ))
        .header("content-type", "application/json")
        .body(
            json!({
                "prompt": "execute a host command through the runtime",
                "session_id": "runtime-node-session"
            })
            .to_string(),
        )
        .send()
        .await
        .expect("send agent run request");

    let status = response.status();
    let body: Value = response
        .json()
        .await
        .expect("agent run response should be JSON");

    let _ = test_server.shutdown();
    {
        let _env_lock = env_lock().lock().expect("env lock");
        env::remove_var("OPENROUTER_API_KEY");
        env::remove_var("MATRIXCLAW_OPENAI_BASE_URL");
        env::remove_var("MATRIXCLAW_LLM_MODEL");
    }

    assert_eq!(
        status.as_u16(),
        200,
        "the gateway should still serve the live runtime request"
    );
    assert_eq!(
        request_count.load(Ordering::SeqCst),
        2,
        "the live runtime should still complete the normal provider turn and tool continuation"
    );
    assert_eq!(
        body.get("final_message").and_then(Value::as_str),
        Some("node-visible-result"),
        "gateway-visible runtime output should still be returned"
    );

    let session_id = body
        .get("session_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .expect("agent run should create a persisted session id");
    let session_path = session_db_path(&home, session_id);
    let storage = SqliteStorage::open(&session_path).expect("open persisted session");
    let transcript = storage.load_transcript().expect("load transcript");

    assert!(
        transcript.iter().any(|entry| {
            entry.kind == DurableTranscriptKind::ToolResult
                && entry.content.contains("node-visible-result")
        }),
        "runtime-visible tool results should preserve the execution-node structured response"
    );
}

fn spawn_fixture_provider(request_count: Arc<AtomicUsize>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture provider");
    let address = listener.local_addr().expect("fixture provider address");

    thread::spawn(move || {
        {
            let (mut stream, _) = listener.accept().expect("accept fixture request");
            let _ = read_http_request(&mut stream);
            request_count.fetch_add(1, Ordering::SeqCst);
            let response = concat!(
                "HTTP/1.1 200 OK\r\n",
                "Content-Type: text/event-stream\r\n",
                "Connection: close\r\n",
                "\r\n",
                "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_1\",\"function\":{\"name\":\"terminal\",\"arguments\":\"{\\\"command\\\":\\\"echo node-visible-result\\\"}\"}}]}}]}\n\n",
                "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
                "data: [DONE]\n\n"
            );
            stream
                .write_all(response.as_bytes())
                .expect("write fixture response");
        }

        {
            let (mut stream, _) = listener.accept().expect("accept fixture request");
            let _ = read_http_request(&mut stream);
            request_count.fetch_add(1, Ordering::SeqCst);
            let response = concat!(
                "HTTP/1.1 200 OK\r\n",
                "Content-Type: text/event-stream\r\n",
                "Connection: close\r\n",
                "\r\n",
                "data: {\"choices\":[{\"delta\":{\"content\":\"node-visible-result\"}}]}\n\n",
                "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
                "data: [DONE]\n\n"
            );
            stream
                .write_all(response.as_bytes())
                .expect("write fixture response");
        }
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
        "zstar-runtime-node-home-{}-{}",
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
