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
use zstar_app_host::live_runtime::session_db_path;
use zstar_app_host::ui_assets::UiAssetLayout;
use zstar_session_runtime::queue::{QueueItem, SessionQueue};
use zstar_session_runtime::recovery::SessionRecoveryStore;
use zstar_session_runtime::session::Session;
use zstar_session_runtime::sqlite::SqliteStorage;
use zstar_session_runtime::storage::TranscriptStore;
use zstar_session_runtime::RuntimeMessage;

#[test]
fn session_resume_over_http() {
    let _env_lock = env_lock().lock().expect("env lock");
    let home = temp_home();
    let session_id = "resume-over-http-fixture";
    let session_path = session_db_path(&home, session_id);
    seed_persisted_session(&session_path);

    let request_count = Arc::new(AtomicUsize::new(0));
    let request_bodies = Arc::new(Mutex::new(Vec::new()));
    let server_url = spawn_fixture_server(request_count.clone(), request_bodies.clone());

    env::set_var("OPENROUTER_API_KEY", "test-key");
    env::set_var("MATRIXCLAW_OPENAI_BASE_URL", &server_url);
    env::set_var("MATRIXCLAW_LLM_MODEL", "moonshotai/kimi-k2.5");

    let surface = SetupSurface::new(&home, UiAssetLayout::discover());
    let response = surface.handle(HttpRequest::post(
        AGENT_RUN_ROUTE,
        json!({
            "prompt": "continue from here",
            "session_id": session_id,
        })
        .to_string(),
    ));

    env::remove_var("OPENROUTER_API_KEY");
    env::remove_var("MATRIXCLAW_OPENAI_BASE_URL");
    env::remove_var("MATRIXCLAW_LLM_MODEL");

    assert_eq!(response.status_code, 200, "HTTP run should succeed");
    assert_eq!(
        request_count.load(Ordering::SeqCst),
        1,
        "the request should reach the fixture provider exactly once"
    );

    let captured_request = request_bodies
        .lock()
        .expect("request bodies lock")
        .first()
        .cloned()
        .expect("fixture request body");
    let request_body = captured_request
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .expect("fixture request should contain headers and body");
    let body: Value = serde_json::from_str(request_body).expect("parse upstream request body");
    let messages = body
        .get("messages")
        .and_then(Value::as_array)
        .expect("provider request should contain chat messages");

    assert!(
        messages.len() >= 3,
        "expected the resumed request to carry the persisted transcript plus steering metadata"
    );
    assert!(
        messages.iter().any(|message| {
            message.get("content").and_then(Value::as_str) == Some("persisted assistant state")
        }),
        "expected persisted assistant history to be included in the resumed request"
    );
    assert!(
        messages.iter().any(|message| {
            message.get("content").and_then(Value::as_str) == Some("keep this steering")
        }),
        "expected steering metadata to survive restart and reach the next prompt"
    );
    assert!(
        !messages.iter().any(|message| {
            message.get("content").and_then(Value::as_str) == Some("defer this follow-up")
        }),
        "follow-up metadata should stay deferred until the current run completes"
    );
    assert!(
        messages.iter().any(|message| {
            message.get("content").and_then(Value::as_str) == Some("continue from here")
        }),
        "expected the new prompt to still be present"
    );

    let resumed_storage = SqliteStorage::open(&session_path).expect("reopen persisted session");
    let transcript = resumed_storage
        .load_transcript()
        .expect("load persisted transcript");
    assert!(
        transcript
            .iter()
            .any(|entry| entry.content == "persisted assistant state"),
        "the previous transcript should remain persisted"
    );
    let recovered = resumed_storage
        .load_recovery_snapshot()
        .expect("load recovery snapshot after resumed run");
    assert_eq!(
        recovered.queue.steering_items().count(),
        0,
        "steering should be drained after the resumed turn"
    );
    assert_eq!(
        recovered.queue.follow_up_items().count(),
        1,
        "follow-up should remain queued for the next run"
    );
}

fn seed_persisted_session(path: &PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create session directory");
    }

    let mut storage = SqliteStorage::open(path).expect("open session storage");
    let session = Session::from_parts(
        vec![RuntimeMessage::Assistant(
            "persisted assistant state".to_string(),
        )],
        SessionQueue::from_items(vec![
            QueueItem::Steering("keep this steering".to_string()),
            QueueItem::FollowUp("defer this follow-up".to_string()),
        ]),
        Vec::new(),
    );

    storage.persist_session(&session).expect("seed session");
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
            "data: {\"choices\":[{\"delta\":{\"content\":\"Persisted \"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n\n",
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
        "zstar-session-resume-home-{}-{}",
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
