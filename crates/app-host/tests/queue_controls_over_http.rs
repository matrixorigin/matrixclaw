use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use zstar_app_host::http::agent_api::AGENT_RUN_ROUTE;
use zstar_app_host::http::{HttpRequest, SetupSurface};
use zstar_app_host::ui_assets::UiAssetLayout;

#[test]
fn queue_controls_over_http_share_the_live_session() {
    let _env_lock = env_lock().lock().expect("env lock");
    let home = temp_home();
    let request_count = Arc::new(AtomicUsize::new(0));
    let request_bodies = Arc::new(Mutex::new(Vec::new()));
    let server_url = spawn_fixture_server(request_count.clone(), request_bodies.clone());

    env::set_var("OPENROUTER_API_KEY", "test-key");
    env::set_var("MATRIXCLAW_OPENAI_BASE_URL", &server_url);
    env::set_var("MATRIXCLAW_LLM_MODEL", "moonshotai/kimi-k2.5");

    let surface = SetupSurface::new(&home, UiAssetLayout::discover());

    let steering_response = surface.handle(HttpRequest::post(
        "/api/queue/steering",
        json!({
            "kind": "steering",
            "message": "prefer the file reference"
        })
        .to_string(),
    ));
    assert_eq!(
        steering_response.status_code, 200,
        "queueing steering should succeed"
    );
    let steering_body: Value =
        serde_json::from_slice(&steering_response.body).expect("steering response JSON");
    let session_id = steering_body
        .get("session_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .expect("queue submission should allocate a session id")
        .to_string();

    let follow_up_response = surface.handle(HttpRequest::post(
        "/api/queue/follow-up",
        json!({
            "kind": "follow-up",
            "message": "check the next turn after this one",
            "session_id": session_id,
        })
        .to_string(),
    ));
    assert_eq!(
        follow_up_response.status_code, 200,
        "queueing follow-up should succeed"
    );

    let queue_state_response = surface.handle(HttpRequest::get(format!(
        "/api/queue/state?session_id={session_id}"
    )));
    assert_eq!(
        queue_state_response.status_code, 200,
        "queue state should load for the session"
    );
    let queue_state: Value =
        serde_json::from_slice(&queue_state_response.body).expect("queue state JSON");
    assert_eq!(
        queue_state["steering"]["summary"].as_str(),
        Some("1 steering item(s) queued for the next assistant turn")
    );
    assert_eq!(
        queue_state["follow_up"]["summary"].as_str(),
        Some("1 follow-up item(s) deferred until the current run completes")
    );

    let first_run_response = surface.handle(HttpRequest::post(
        AGENT_RUN_ROUTE,
        json!({
            "prompt": "start the run",
            "session_id": session_id,
        })
        .to_string(),
    ));
    assert_eq!(
        first_run_response.status_code, 200,
        "first run should succeed"
    );

    let second_run_response = surface.handle(HttpRequest::post(
        AGENT_RUN_ROUTE,
        json!({
            "prompt": "continue the run",
            "session_id": session_id,
        })
        .to_string(),
    ));
    assert_eq!(
        second_run_response.status_code, 200,
        "second run should succeed"
    );

    env::remove_var("OPENROUTER_API_KEY");
    env::remove_var("MATRIXCLAW_OPENAI_BASE_URL");
    env::remove_var("MATRIXCLAW_LLM_MODEL");

    assert_eq!(
        request_count.load(Ordering::SeqCst),
        2,
        "each HTTP run should make exactly one provider request"
    );

    let captured = request_bodies.lock().expect("request bodies lock");
    assert_eq!(
        captured.len(),
        2,
        "two upstream requests should have been recorded"
    );

    let first_messages = extract_message_contents(&captured[0]);
    assert!(first_messages.contains(&"prefer the file reference".to_string()));
    assert!(!first_messages.contains(&"check the next turn after this one".to_string()));
    assert!(first_messages.contains(&"start the run".to_string()));

    let second_messages = extract_message_contents(&captured[1]);
    assert!(!second_messages.contains(&"prefer the file reference".to_string()));
    assert!(second_messages.contains(&"check the next turn after this one".to_string()));
    assert!(second_messages.contains(&"continue the run".to_string()));
}

fn extract_message_contents(raw_request: &str) -> Vec<String> {
    let request_body = raw_request
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .expect("fixture request should contain headers and body");
    let payload: Value = serde_json::from_str(request_body).expect("parse upstream request body");
    payload["messages"]
        .as_array()
        .expect("provider request should contain chat messages")
        .iter()
        .filter_map(|message| message.get("content").and_then(Value::as_str))
        .map(str::to_string)
        .collect()
}

fn spawn_fixture_server(
    request_count: Arc<AtomicUsize>,
    request_bodies: Arc<Mutex<Vec<String>>>,
) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture server");
    let address = listener.local_addr().expect("fixture server addr");

    thread::spawn(move || {
        for _ in 0..2 {
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
                "data: {\"choices\":[{\"delta\":{\"content\":\"Persisted \"}}]}\n\n",
                "data: {\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n\n",
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
        "zstar-queue-over-http-home-{}-{}",
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
