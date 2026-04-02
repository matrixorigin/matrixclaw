use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_app_host::http::SetupSurface;
use matrixclaw_app_host::live_runtime::session_db_path;
use matrixclaw_app_host::server::spawn_test_server;
use matrixclaw_app_host::ui_assets::UiAssetLayout;
use serde_json::{json, Value};

#[tokio::test]
async fn cross_transport_session_reuse() {
    let home = temp_home();
    let request_count = Arc::new(AtomicUsize::new(0));
    let request_bodies = Arc::new(Mutex::new(Vec::new()));
    let server_url = spawn_fixture_server(request_count.clone(), request_bodies.clone());

    {
        let _env_lock = env_lock().lock().expect("env lock");
        env::set_var("OPENROUTER_API_KEY", "test-key");
        env::set_var("MATRIXCLAW_OPENAI_BASE_URL", &server_url);
        env::set_var("MATRIXCLAW_LLM_MODEL", "moonshotai/kimi-k2.5");
    }

    let surface = SetupSurface::new(&home, UiAssetLayout::discover());
    let test_server = spawn_test_server(surface).expect("spawn test server");
    let client = reqwest::Client::new();
    let session_id = "cross-transport-session";

    let browser_response = client
        .post(format!("http://{}/api/agent/run", test_server.address))
        .header("content-type", "application/json")
        .body(
            json!({
                "prompt": "seed through the browser transport",
                "session_id": session_id,
            })
            .to_string(),
        )
        .send()
        .await
        .expect("send browser request");
    assert_eq!(
        browser_response.status().as_u16(),
        200,
        "browser route should succeed"
    );
    let browser_body: Value = browser_response
        .json()
        .await
        .expect("browser response JSON");
    assert_eq!(
        browser_body.get("session_id").and_then(Value::as_str),
        Some(session_id),
        "browser route should reuse the explicit session id"
    );

    let steering_response = client
        .post(format!("http://{}/api/queue/steering", test_server.address))
        .header("content-type", "application/json")
        .body(
            json!({
                "kind": "steering",
                "session_id": session_id,
                "message": "cross transport steering"
            })
            .to_string(),
        )
        .send()
        .await
        .expect("queue steering");
    assert_eq!(
        steering_response.status().as_u16(),
        200,
        "steering should be accepted"
    );

    let follow_up_response = client
        .post(format!(
            "http://{}/api/queue/follow-up",
            test_server.address
        ))
        .header("content-type", "application/json")
        .body(
            json!({
                "kind": "follow-up",
                "session_id": session_id,
                "message": "defer across transports"
            })
            .to_string(),
        )
        .send()
        .await
        .expect("queue follow up");
    assert_eq!(
        follow_up_response.status().as_u16(),
        200,
        "follow-up should be accepted"
    );

    let openclaw_response = client
        .post(format!("http://{}/api/openclaw/chat", test_server.address))
        .header("content-type", "application/json")
        .body(
            json!({
                "conversation_id": session_id,
                "messages": [
                    { "role": "user", "content": "resume through the openclaw transport" }
                ]
            })
            .to_string(),
        )
        .send()
        .await
        .expect("send openclaw request");

    {
        let _env_lock = env_lock().lock().expect("env lock");
        env::remove_var("OPENROUTER_API_KEY");
        env::remove_var("MATRIXCLAW_OPENAI_BASE_URL");
        env::remove_var("MATRIXCLAW_LLM_MODEL");
    }

    assert_eq!(
        openclaw_response.status().as_u16(),
        200,
        "openclaw route should succeed"
    );
    let openclaw_body: Value = openclaw_response
        .json()
        .await
        .expect("openclaw response JSON");
    assert_eq!(
        openclaw_body.get("conversation_id").and_then(Value::as_str),
        Some(session_id),
        "OpenClaw route should resume the same persisted session id"
    );

    assert_eq!(
        request_count.load(Ordering::SeqCst),
        2,
        "browser and OpenClaw transports should each hit the provider once"
    );
    assert!(
        session_db_path(&home, session_id).exists(),
        "cross-transport reuse should persist one shared session store"
    );

    let captured_requests = request_bodies.lock().expect("request body lock");
    let second_request = captured_requests
        .get(1)
        .cloned()
        .expect("captured openclaw provider request");
    let second_body = second_request
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .expect("provider request should contain a body");
    let upstream: Value = serde_json::from_str(second_body).expect("parse provider request JSON");
    let messages = upstream
        .get("messages")
        .and_then(Value::as_array)
        .expect("provider request should contain messages");

    assert!(
        messages.iter().any(|message| {
            message.get("content").and_then(Value::as_str) == Some("browser seeded reply")
        }),
        "OpenClaw resume should see the persisted browser assistant history"
    );
    assert!(
        messages.iter().any(|message| {
            message.get("content").and_then(Value::as_str) == Some("cross transport steering")
        }),
        "OpenClaw resume should apply steering queued through the browser boundary"
    );
    assert!(
        !messages.iter().any(|message| {
            message.get("content").and_then(Value::as_str) == Some("defer across transports")
        }),
        "follow-up should remain deferred for the next run even across transports"
    );
    assert!(
        messages.iter().any(|message| {
            message.get("content").and_then(Value::as_str)
                == Some("resume through the openclaw transport")
        }),
        "OpenClaw user content should still reach the shared runtime"
    );

    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonical repo root");
    assert!(
        repo_root
            .join("scripts/verify-served-transports.sh")
            .exists(),
        "task 005 requires a maintainer-facing served transport harness"
    );

    test_server.shutdown().expect("shutdown server");
}

fn spawn_fixture_server(
    request_count: Arc<AtomicUsize>,
    request_bodies: Arc<Mutex<Vec<String>>>,
) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture server");
    let address = listener.local_addr().expect("fixture server addr");

    thread::spawn(move || {
        while let Ok((mut stream, _)) = listener.accept() {
            let request = read_http_request(&mut stream);
            let current = request_count.fetch_add(1, Ordering::SeqCst);
            request_bodies
                .lock()
                .expect("lock request body sink")
                .push(request);

            let assistant = if current == 0 {
                "browser seeded reply"
            } else {
                "openclaw resumed reply"
            };

            let response = format!(
                concat!(
                    "HTTP/1.1 200 OK\r\n",
                    "Content-Type: text/event-stream\r\n",
                    "Connection: close\r\n",
                    "\r\n",
                    "data: {{\"choices\":[{{\"delta\":{{\"content\":\"{assistant}\"}}}}]}}\n\n",
                    "data: {{\"choices\":[{{\"delta\":{{}},\"finish_reason\":\"stop\"}}]}}\n\n",
                    "data: [DONE]\n\n"
                ),
                assistant = assistant
            );

            stream
                .write_all(response.as_bytes())
                .expect("write fixture response");

            if current >= 1 {
                break;
            }
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

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!(
        "matrixclaw-cross-transport-home-{}-{}",
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
