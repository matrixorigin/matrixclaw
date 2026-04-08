use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use zstar_agent_core::event::AgentEvent;
use zstar_agent_core::provider::Provider;
use zstar_agent_core::RunRequest;
use zstar_app_host::openai_compatible::OpenAiCompatibleProvider;

#[tokio::test]
async fn openrouter_provider_streaming() {
    let request_count = Arc::new(AtomicUsize::new(0));
    let request_bodies = Arc::new(Mutex::new(Vec::new()));
    let server_url = spawn_fixture_server(request_count.clone(), request_bodies.clone());

    let mut provider =
        OpenAiCompatibleProvider::with_base_url(server_url, "test-key", "moonshotai/kimi-k2.5")
            .expect("create fixture-backed provider");

    let request = RunRequest::new("Say ZStar");
    let mut events = Vec::new();

    let result = provider
        .stream(&request, &mut |event| events.push(event))
        .await;

    let body = request_bodies
        .lock()
        .expect("lock request bodies")
        .first()
        .cloned()
        .expect("fixture request body");

    assert_eq!(
        request_count.load(Ordering::SeqCst),
        1,
        "provider should make exactly one upstream call for a streamed turn"
    );
    assert!(
        body.contains("\"stream\":true"),
        "provider should request streaming instead of probing with a non-stream completion: {body}"
    );

    let streamed = result.expect("streaming fixture response should parse");
    assert_eq!(streamed.content.as_deref(), Some("ZStar"));
    assert_eq!(
        events,
        vec![
            AgentEvent::MessageDelta("Z".to_string()),
            AgentEvent::MessageDelta("Star".to_string()),
        ],
        "streamed provider boundary should emit ordered deltas"
    );
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
            "data: {\"choices\":[{\"delta\":{\"content\":\"Z\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"Star\"}}]}\n\n",
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
