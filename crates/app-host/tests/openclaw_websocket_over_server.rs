use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use serde_json::{json, Value};
use zstar_app_host::http::SetupSurface;
use zstar_app_host::live_runtime::session_db_path;
use zstar_app_host::server::spawn_test_server;
use zstar_app_host::ui_assets::UiAssetLayout;

#[test]
fn openclaw_websocket_over_server() {
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
    let conversation_id = "openclaw-ws-session";
    let mut socket = websocket_connect(test_server.address);

    let challenge = read_text_frame(&mut socket);
    let authenticated = read_text_frame(&mut socket);
    let agents_list = read_text_frame(&mut socket);

    assert_eq!(
        challenge.get("type").and_then(Value::as_str),
        Some("challenge"),
        "served websocket should begin with a challenge frame"
    );
    assert_eq!(
        authenticated.get("type").and_then(Value::as_str),
        Some("authenticated"),
        "served websocket should report successful protocol authentication"
    );
    assert_eq!(
        agents_list.get("type").and_then(Value::as_str),
        Some("agents_list"),
        "served websocket should report available agents before chat"
    );

    let request = json!({
        "type": "chat",
        "conversation_id": conversation_id,
        "messages": [
            { "role": "system", "content": "stay protocol-shaped" },
            { "role": "user", "content": "route over served websocket" }
        ]
    });
    write_text_frame(&mut socket, &request.to_string());

    let assistant = read_text_frame(&mut socket);
    let completed = read_text_frame(&mut socket);

    env::remove_var("OPENROUTER_API_KEY");
    env::remove_var("MATRIXCLAW_OPENAI_BASE_URL");
    env::remove_var("MATRIXCLAW_LLM_MODEL");

    assert_eq!(
        assistant.get("type").and_then(Value::as_str),
        Some("assistant_chunk"),
        "served websocket should emit compatibility-shaped assistant output"
    );
    assert_eq!(
        assistant.get("content").and_then(Value::as_str),
        Some("compatibility answer"),
        "served websocket should project the shared runtime completion"
    );
    assert_eq!(
        completed.get("type").and_then(Value::as_str),
        Some("completed"),
        "served websocket should terminate the conversation with a completed frame"
    );
    assert_eq!(
        request_count.load(Ordering::SeqCst),
        1,
        "served websocket should hit the provider exactly once"
    );
    assert!(
        session_db_path(&home, conversation_id).exists(),
        "served websocket transport should persist to the shared session store"
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
            message.get("content").and_then(Value::as_str) == Some("stay protocol-shaped")
        }),
        "system context should reach the shared runtime over websocket"
    );
    assert!(
        messages.iter().any(|message| {
            message.get("content").and_then(Value::as_str) == Some("route over served websocket")
        }),
        "user message should reach the shared runtime over websocket"
    );

    test_server.shutdown().expect("shutdown server");
}

fn websocket_connect(address: std::net::SocketAddr) -> TcpStream {
    let mut stream = TcpStream::connect(address).expect("connect websocket");
    let request = format!(
        concat!(
            "GET /api/openclaw/ws HTTP/1.1\r\n",
            "Host: {host}\r\n",
            "Upgrade: websocket\r\n",
            "Connection: Upgrade\r\n",
            "Sec-WebSocket-Version: 13\r\n",
            "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n",
            "\r\n"
        ),
        host = address
    );
    stream
        .write_all(request.as_bytes())
        .expect("write websocket handshake");

    let response = read_http_response_head(&mut stream);
    assert!(
        response.starts_with("HTTP/1.1 101"),
        "expected websocket upgrade response, got {response:?}"
    );
    stream
}

fn read_http_response_head(stream: &mut TcpStream) -> String {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        let read = stream.read(&mut chunk).expect("read handshake response");
        assert!(read > 0, "handshake closed before response head completed");
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }
    String::from_utf8(buffer).expect("handshake response should be utf-8")
}

fn write_text_frame(stream: &mut TcpStream, payload: &str) {
    let mut frame = Vec::with_capacity(payload.len() + 8);
    frame.push(0x81);
    let payload_bytes = payload.as_bytes();
    if payload_bytes.len() < 126 {
        frame.push(0x80 | payload_bytes.len() as u8);
    } else {
        frame.push(0x80 | 126);
        frame.extend_from_slice(&(payload_bytes.len() as u16).to_be_bytes());
    }
    let mask = [0x37, 0xfa, 0x21, 0x3d];
    frame.extend_from_slice(&mask);
    for (index, byte) in payload_bytes.iter().enumerate() {
        frame.push(byte ^ mask[index % 4]);
    }
    stream.write_all(&frame).expect("write websocket frame");
}

fn read_text_frame(stream: &mut TcpStream) -> Value {
    let mut header = [0_u8; 2];
    stream.read_exact(&mut header).expect("read frame header");

    let opcode = header[0] & 0x0f;
    assert_eq!(opcode, 0x1, "expected text frame");

    let masked = (header[1] & 0x80) != 0;
    assert!(!masked, "server text frame should not be masked");

    let mut len = (header[1] & 0x7f) as usize;
    if len == 126 {
        let mut extended = [0_u8; 2];
        stream
            .read_exact(&mut extended)
            .expect("read extended payload length");
        len = u16::from_be_bytes(extended) as usize;
    }

    let mut payload = vec![0_u8; len];
    stream.read_exact(&mut payload).expect("read frame payload");
    serde_json::from_slice(&payload).expect("parse websocket JSON frame")
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
        "zstar-openclaw-ws-{}-{}",
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

    format!("http://{address}")
}

fn read_http_request(stream: &mut TcpStream) -> String {
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
