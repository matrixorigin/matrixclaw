use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use zstar_app_host::http::SetupSurface;
use zstar_app_host::server::spawn_test_server;
use zstar_app_host::ui_assets::UiAssetLayout;

#[test]
fn openclaw_streaming_parity() {
    let _env_lock = env_lock().lock().expect("env lock");
    let home = temp_home();
    let server_url = spawn_slow_fixture_server();

    env::set_var("OPENROUTER_API_KEY", "test-key");
    env::set_var("MATRIXCLAW_OPENAI_BASE_URL", &server_url);
    env::set_var("MATRIXCLAW_LLM_MODEL", "moonshotai/kimi-k2.5");

    let surface = SetupSurface::new(&home, UiAssetLayout::discover());
    let test_server = spawn_test_server(surface).expect("spawn test server");
    let mut socket = websocket_connect(test_server.address);

    let _challenge = read_text_frame(&mut socket);
    let _authenticated = read_text_frame(&mut socket);
    let _agents_list = read_text_frame(&mut socket);

    let request = json!({
        "type": "chat",
        "conversation_id": "openclaw-streaming-session",
        "messages": [
            { "role": "user", "content": "stream progressively" }
        ]
    });
    write_text_frame(&mut socket, &request.to_string());

    let started = Instant::now();
    let first = read_text_frame(&mut socket);
    let first_elapsed = started.elapsed();
    let second = read_text_frame(&mut socket);
    let completed = read_text_frame(&mut socket);

    env::remove_var("OPENROUTER_API_KEY");
    env::remove_var("MATRIXCLAW_OPENAI_BASE_URL");
    env::remove_var("MATRIXCLAW_LLM_MODEL");

    assert!(
        first_elapsed < Duration::from_millis(150),
        "first assistant frame should arrive before the upstream stream finishes, got {first_elapsed:?}"
    );
    assert_eq!(
        first.get("type").and_then(Value::as_str),
        Some("assistant_chunk")
    );
    assert_eq!(first.get("content").and_then(Value::as_str), Some("first "));
    assert_eq!(
        second.get("type").and_then(Value::as_str),
        Some("assistant_chunk")
    );
    assert_eq!(
        second.get("content").and_then(Value::as_str),
        Some("second")
    );
    assert_eq!(
        completed.get("type").and_then(Value::as_str),
        Some("completed")
    );

    test_server.shutdown().expect("shutdown server");
}

fn env_lock() -> &'static std::sync::Mutex<()> {
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    &ENV_LOCK
}

fn temp_home() -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!(
        "zstar-openclaw-streaming-{}-{}",
        std::process::id(),
        nanos
    ));
    std::fs::create_dir_all(&home).expect("create temp home");
    home
}

fn spawn_slow_fixture_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture server");
    let address = listener.local_addr().expect("fixture server addr");

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept fixture request");
        let _request = read_http_request(&mut stream);

        let response_head = concat!(
            "HTTP/1.1 200 OK\r\n",
            "Content-Type: text/event-stream\r\n",
            "Connection: close\r\n",
            "\r\n"
        );
        stream
            .write_all(response_head.as_bytes())
            .expect("write fixture head");

        stream
            .write_all(b"data: {\"choices\":[{\"delta\":{\"content\":\"first \"}}]}\n\n")
            .expect("write first delta");
        stream.flush().expect("flush first delta");
        thread::sleep(Duration::from_millis(250));
        stream
            .write_all(b"data: {\"choices\":[{\"delta\":{\"content\":\"second\"}}]}\n\n")
            .expect("write second delta");
        stream
            .write_all(b"data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n")
            .expect("write finish delta");
        stream.write_all(b"data: [DONE]\n\n").expect("write done");
        stream.flush().expect("flush done");
    });

    format!("http://{address}")
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
        if let Some(end) = find_header_end(&buffer) {
            return String::from_utf8(buffer[..end].to_vec())
                .expect("handshake response should be utf-8");
        }
    }
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

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
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
