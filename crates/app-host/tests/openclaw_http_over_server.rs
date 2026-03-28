use std::env;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use matrixclaw_app_host::live_runtime::session_db_path;
use matrixclaw_app_host::server::spawn_test_server;
use matrixclaw_app_host::http::SetupSurface;
use matrixclaw_app_host::ui_assets::UiAssetLayout;
use serde_json::{json, Value};

#[test]
fn openclaw_http_over_server() {
    let _env_lock = env_lock().lock().expect("env lock");
    let home = temp_home();
    let request_count = Arc::new(AtomicUsize::new(0));
    let request_bodies = Arc::new(Mutex::new(Vec::new()));
    let server_url = spawn_fixture_server(request_count.clone(), request_bodies.clone());

    env::set_var("OPENROUTER_API_KEY", "test-key");
    env::set_var("MATRIXCLAW_OPENAI_BASE_URL", &server_url);
    env::set_var("MATRIXCLAW_LLM_MODEL", "moonshotai/kimi-k2.5");

    let surface = SetupSurface::new(&home, UiAssetLayout::discover());
    let test_server = spawn_test_server(surface).expect("spawn test server");
    let conversation_id = "openclaw-http-session";
    let response = reqwest::blocking::Client::new()
        .post(format!(
            "http://{}/api/openclaw/chat",
            test_server.address
        ))
        .header("content-type", "application/json")
        .body(
            json!({
                "conversation_id": conversation_id,
                "messages": [
                    { "role": "system", "content": "keep transport protocol-shaped" },
                    { "role": "user", "content": "route over served http" }
                ]
            })
            .to_string(),
        )
        .send()
        .expect("send openclaw http request");

    env::remove_var("OPENROUTER_API_KEY");
    env::remove_var("MATRIXCLAW_OPENAI_BASE_URL");
    env::remove_var("MATRIXCLAW_LLM_MODEL");

    assert_eq!(response.status(), 200, "served OpenClaw HTTP endpoint should succeed");

    let body: Value = response.json().expect("response JSON");
    assert_eq!(
        body.get("conversation_id").and_then(Value::as_str),
        Some(conversation_id),
        "served response should preserve the compatibility conversation id"
    );
    assert_eq!(
        body.pointer("/frames/0/content").and_then(Value::as_str),
        Some("compatibility answer"),
        "served response should contain compatibility-shaped assistant frames"
    );

    assert_eq!(
        request_count.load(Ordering::SeqCst),
        1,
        "served transport should hit the provider boundary once"
    );
    assert!(
        session_db_path(&home, conversation_id).exists(),
        "served OpenClaw HTTP transport should persist to the shared browser session store"
    );

    let captured_request = request_bodies
        .lock()
        .expect("request bodies lock")
        .first()
        .cloned()
        .expect("captured provider request");
    let request_body = captured_request
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .expect("provider request should contain a body");
    let upstream: Value = serde_json::from_str(request_body).expect("parse provider request JSON");
    let messages = upstream
        .get("messages")
        .and_then(Value::as_array)
        .expect("provider request should contain messages");
    assert!(
        messages.iter().any(|message| {
            message.get("content").and_then(Value::as_str)
                == Some("keep transport protocol-shaped")
        }),
        "served OpenClaw system context should enter the shared runtime request"
    );
    assert!(
        messages.iter().any(|message| {
            message.get("content").and_then(Value::as_str) == Some("route over served http")
        }),
        "served OpenClaw user turn should enter the shared runtime request"
    );

    test_server.shutdown().expect("shutdown server");
}

fn env_lock() -> &'static Mutex<()> {
    static ENV_LOCK: Mutex<()> = Mutex::new(());
    &ENV_LOCK
}

fn temp_home() -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!(
        "matrixclaw-openclaw-http-{}-{}",
        std::process::id(),
        nanos
    ));
    std::fs::create_dir_all(&home).expect("create temp home");
    home
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
            "data: {\"choices\":[{\"delta\":{\"content\":\"compatibility answer\"}}]}\n\n",
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
    let headers = String::from_utf8_lossy(headers);
    headers
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
