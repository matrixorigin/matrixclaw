use std::io::{self, Read, Write};

use base64::Engine;
use serde::Deserialize;
use serde_json::json;
use sha1::{Digest, Sha1};
use tiny_http::{Header, Request, Response, StatusCode};

use crate::http::agent_api::{build_provider_from_env, resolve_model};
use crate::http::{HttpRequest, HttpResponse, SetupSurface};
use crate::openclaw_transport;
use matrixclaw_compat_openclaw::websocket::openclaw_agents_list;
use matrixclaw_compat_openclaw::translation::{
    OpenClawChatMessage, OpenClawChatRequest, OpenClawChatRole,
};

pub const OPENCLAW_CHAT_ROUTE: &str = "/api/openclaw/chat";
pub const OPENCLAW_WEBSOCKET_ROUTE: &str = "/api/openclaw/ws";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OpenClawHttpRequest {
    pub conversation_id: String,
    pub messages: Vec<OpenClawHttpMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OpenClawHttpMessage {
    pub role: String,
    pub content: String,
}

pub fn is_openclaw_chat_route(path: &str) -> bool {
    crate::http::routes::normalize_path(path) == OPENCLAW_CHAT_ROUTE
}

pub fn is_openclaw_websocket_route(path: &str) -> bool {
    crate::http::routes::normalize_path(path) == OPENCLAW_WEBSOCKET_ROUTE
}

pub fn openclaw_chat_response(surface: &SetupSurface, request: HttpRequest) -> HttpResponse {
    let payload = match parse_openclaw_request(&request.body) {
        Ok(payload) => payload,
        Err(response) => return response,
    };

    let model = resolve_model(surface);
    let mut provider = match build_provider_from_env(surface, &model) {
        Ok(provider) => provider,
        Err(response) => return response,
    };

    let response = match openclaw_transport::openclaw_chat_http(
        surface.home(),
        model,
        &payload,
        &mut provider,
    ) {
        Ok(response) => response,
        Err(error) => {
            return HttpResponse::json(502, json!({ "error": error }).to_string());
        }
    };

    let body = serde_json::to_string_pretty(&response).expect("serialize OpenClaw chat response");
    HttpResponse::json(200, body)
}

pub fn is_websocket_upgrade(request: &Request) -> bool {
    request
        .headers()
        .iter()
        .find(|header| header.field.equiv("Upgrade"))
        .is_some_and(|header| header.value.as_str().eq_ignore_ascii_case("websocket"))
}

pub fn serve_openclaw_websocket(surface: SetupSurface, request: Request) -> io::Result<()> {
    let Some(key) = request
        .headers()
        .iter()
        .find(|header| header.field.equiv("Sec-WebSocket-Key"))
        .map(|header| header.value.as_str().to_string())
    else {
        request
            .respond(Response::new_empty(StatusCode(400)))
            .map_err(io::Error::other)?;
        return Ok(());
    };

    let response = Response::new_empty(StatusCode(101))
        .with_header(
            Header::from_bytes("Upgrade", "websocket")
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid upgrade"))?,
        )
        .with_header(
            Header::from_bytes("Connection", "Upgrade")
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid connection"))?,
        )
        .with_header(
            Header::from_bytes("Sec-WebSocket-Accept", websocket_accept(&key))
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid accept"))?,
        );
    let mut stream = request.upgrade("websocket", response);

    let agents = openclaw_agents_list(true);
    for frame in agents.frames {
        write_websocket_json_frame(&mut stream, &frame)?;
    }

    let payload = read_websocket_text_frame(&mut stream)?;
    let request = parse_openclaw_request(&payload).map_err(|response| io_error_from_response(&response))?;
    let model = resolve_model(&surface);
    let mut provider =
        build_provider_from_env(&surface, &model).map_err(|response| io_error_from_response(&response))?;
    let mut write_error = None;
    let conversation = openclaw_transport::stream_openclaw_chat_websocket(
        surface.home(),
        model,
        &request,
        &mut provider,
        &mut |frame| {
            if write_error.is_none() {
                if let Err(error) = write_websocket_json_frame(&mut stream, &frame) {
                    write_error = Some(error);
                }
            }
        },
    )
    .map_err(io::Error::other)?;

    if let Some(error) = write_error {
        return Err(error);
    }

    if !matches!(
        conversation.frames.last(),
        Some(matrixclaw_compat_openclaw::stream_adapter::ChatFrame::Completed)
    ) {
        write_websocket_json_frame(
            &mut stream,
            &matrixclaw_compat_openclaw::stream_adapter::ChatFrame::Completed,
        )?;
    }

    Ok(())
}

fn parse_openclaw_request(body: &[u8]) -> Result<OpenClawChatRequest, HttpResponse> {
    let Ok(payload) = serde_json::from_slice::<OpenClawHttpRequest>(body) else {
        return Err(HttpResponse::json(
            400,
            json!({ "error": "OpenClaw payload must be valid JSON" }).to_string(),
        ));
    };

    parse_openclaw_payload(payload)
}

fn parse_role(role: &str) -> Option<OpenClawChatRole> {
    match role.trim().to_ascii_lowercase().as_str() {
        "user" => Some(OpenClawChatRole::User),
        "system" => Some(OpenClawChatRole::System),
        "tool" => Some(OpenClawChatRole::Tool),
        "assistant" => Some(OpenClawChatRole::Assistant),
        _ => None,
    }
}

fn parse_openclaw_payload(payload: OpenClawHttpRequest) -> Result<OpenClawChatRequest, HttpResponse> {
    let conversation_id = payload.conversation_id.trim().to_string();
    if conversation_id.is_empty() {
        return Err(HttpResponse::json(
            400,
            json!({ "error": "conversation_id is required" }).to_string(),
        ));
    }

    if payload.messages.is_empty() {
        return Err(HttpResponse::json(
            400,
            json!({ "error": "messages must not be empty" }).to_string(),
        ));
    }

    let mut messages = Vec::with_capacity(payload.messages.len());
    for message in payload.messages {
        let role = parse_role(&message.role).ok_or_else(|| {
            HttpResponse::json(
                400,
                json!({ "error": format!("unsupported OpenClaw role: {}", message.role) })
                    .to_string(),
            )
        })?;

        if message.content.trim().is_empty() {
            return Err(HttpResponse::json(
                400,
                json!({ "error": "message content must not be empty" }).to_string(),
            ));
        }

        messages.push(OpenClawChatMessage {
            role,
            content: message.content,
        });
    }

    Ok(OpenClawChatRequest {
        conversation_id,
        messages,
    })
}

fn websocket_accept(key: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    base64::engine::general_purpose::STANDARD.encode(hasher.finalize())
}

fn read_websocket_text_frame(stream: &mut (impl Read + Write)) -> io::Result<Vec<u8>> {
    let mut header = [0_u8; 2];
    stream.read_exact(&mut header)?;

    let mut len = (header[1] & 0x7f) as usize;
    if len == 126 {
        let mut extended = [0_u8; 2];
        stream.read_exact(&mut extended)?;
        len = u16::from_be_bytes(extended) as usize;
    }

    let masked = (header[1] & 0x80) != 0;
    let mut mask = [0_u8; 4];
    if masked {
        stream.read_exact(&mut mask)?;
    }

    let mut payload = vec![0_u8; len];
    stream.read_exact(&mut payload)?;
    if masked {
        for (index, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[index % 4];
        }
    }

    Ok(payload)
}

fn write_websocket_json_frame(
    stream: &mut (impl Read + Write),
    frame: &impl serde::Serialize,
) -> io::Result<()> {
    let payload = serde_json::to_vec(frame).map_err(io::Error::other)?;
    let mut header = vec![0x81];
    if payload.len() < 126 {
        header.push(payload.len() as u8);
    } else {
        header.push(126);
        header.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    }

    stream.write_all(&header)?;
    stream.write_all(&payload)?;
    stream.flush()
}

fn io_error_from_response(response: &HttpResponse) -> io::Error {
    io::Error::other(response.body_text())
}
