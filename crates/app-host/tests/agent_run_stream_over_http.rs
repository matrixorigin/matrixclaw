use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_app_host::http::SetupSurface;
use matrixclaw_app_host::server::spawn_test_server;
use matrixclaw_app_host::ui_assets::UiAssetLayout;
use serde_json::Value;

#[test]
fn agent_run_stream_over_http() {
    let _env_lock = env_lock().lock().expect("env lock");
    let home = temp_home();
    let server_url = spawn_fixture_server();

    env::set_var("OPENROUTER_API_KEY", "test-key");
    env::set_var("MATRIXCLAW_OPENAI_BASE_URL", &server_url);
    env::set_var("MATRIXCLAW_LLM_MODEL", "moonshotai/kimi-k2.5");

    let surface = SetupSurface::new(&home, UiAssetLayout::discover());
    let test_server = spawn_test_server(surface).expect("spawn test server");
    let response = reqwest::blocking::Client::new()
        .post(format!(
            "http://{}/api/agent/run/stream",
            test_server.address
        ))
        .header("content-type", "application/json")
        .body(r#"{"prompt":"stream the assistant reply","session_id":"stream-http-session"}"#)
        .send()
        .expect("stream request");
    let body = response.text().expect("stream body");

    let _ = test_server.shutdown();
    env::remove_var("OPENROUTER_API_KEY");
    env::remove_var("MATRIXCLAW_OPENAI_BASE_URL");
    env::remove_var("MATRIXCLAW_LLM_MODEL");

    let frames = parse_sse_frames(&body);
    assert!(
        frames.iter().any(|frame| {
            frame.get("type").and_then(Value::as_str) == Some("event")
                && frame
                    .get("event")
                    .and_then(Value::as_object)
                    .and_then(|event| event.get("kind"))
                    .and_then(Value::as_str)
                    == Some("message_delta")
        }),
        "expected streamed runtime events in SSE output: {body}"
    );
    assert!(
        frames.iter().any(|frame| {
            frame.get("type").and_then(Value::as_str) == Some("complete")
                && frame.get("session_id").and_then(Value::as_str) == Some("stream-http-session")
                && frame.get("final_message").and_then(Value::as_str) == Some("Persisted hello")
        }),
        "expected terminal SSE completion frame in output: {body}"
    );
}

fn parse_sse_frames(body: &str) -> Vec<Value> {
    body.split("\n\n")
        .filter_map(|block| {
            let payload = block
                .lines()
                .filter_map(|line| line.strip_prefix("data: "))
                .collect::<Vec<_>>()
                .join("\n");
            if payload.is_empty() {
                None
            } else {
                Some(serde_json::from_str::<Value>(&payload).expect("parse SSE JSON frame"))
            }
        })
        .collect()
}

fn spawn_fixture_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture server");
    let address = listener.local_addr().expect("fixture server addr");

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept fixture request");
        let _request = read_http_request(&mut stream);

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
        "matrixclaw-stream-http-home-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&home).expect("create temp home");
    home
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}
