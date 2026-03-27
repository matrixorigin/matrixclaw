use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::http::{HttpRequest, HttpResponse, SetupSurface};
use crate::live_runtime::{
    load_or_create_session_for_request, load_session_queue, persist_session_for_id,
};
use matrixclaw_session_runtime::queue::SessionQueue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QueueControlKind {
    Steering,
    FollowUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QueueDeliveryTiming {
    NextTurn,
    NextRun,
    Queued,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueControlState {
    pub kind: QueueControlKind,
    pub submit_route: String,
    pub delivery_timing: QueueDeliveryTiming,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueControlsView {
    pub steering: QueueControlState,
    pub follow_up: QueueControlState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueControlsContract {
    pub steering_submit_route: String,
    pub follow_up_submit_route: String,
    pub state_route: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueSubmissionRequest {
    pub kind: QueueControlKind,
    pub message: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueSubmissionResult {
    pub accepted: bool,
    pub session_id: String,
    pub state: QueueControlState,
}

pub fn queue_controls_contract() -> QueueControlsContract {
    QueueControlsContract {
        steering_submit_route: "/api/queue/steering".to_string(),
        follow_up_submit_route: "/api/queue/follow-up".to_string(),
        state_route: "/api/queue/state".to_string(),
    }
}

pub fn is_queue_state_route(path: &str) -> bool {
    crate::http::routes::normalize_path(path) == "/api/queue/state"
}

pub fn is_queue_submit_route(path: &str) -> bool {
    matches!(
        crate::http::routes::normalize_path(path).as_str(),
        "/api/queue/steering" | "/api/queue/follow-up"
    )
}

pub fn queue_controls_view(queue: &SessionQueue) -> QueueControlsView {
    let contract = queue_controls_contract();
    let steering_count = queue.steering_items().count();
    let follow_up_count = queue.follow_up_items().count();

    QueueControlsView {
        steering: QueueControlState {
            kind: QueueControlKind::Steering,
            submit_route: contract.steering_submit_route,
            delivery_timing: QueueDeliveryTiming::NextTurn,
            summary: format!(
                "{steering_count} steering item(s) queued for the next assistant turn"
            ),
        },
        follow_up: QueueControlState {
            kind: QueueControlKind::FollowUp,
            submit_route: contract.follow_up_submit_route,
            delivery_timing: QueueDeliveryTiming::NextRun,
            summary: format!(
                "{follow_up_count} follow-up item(s) deferred until the current run completes"
            ),
        },
    }
}

pub fn submit_queue_control(
    queue: &mut SessionQueue,
    request: QueueSubmissionRequest,
) -> QueueSubmissionResult {
    match request.kind {
        QueueControlKind::Steering => queue.push_steering(request.message),
        QueueControlKind::FollowUp => queue.push_follow_up(request.message),
    }

    let view = queue_controls_view(queue);
    let state = match request.kind {
        QueueControlKind::Steering => view.steering,
        QueueControlKind::FollowUp => view.follow_up,
    };

    QueueSubmissionResult {
        accepted: true,
        session_id: request.session_id.unwrap_or_default(),
        state,
    }
}

pub fn queue_state_response(surface: &SetupSurface, request: &HttpRequest) -> HttpResponse {
    let queue =
        match load_session_queue(surface.home(), request_session_id(&request.path).as_deref()) {
            Ok(queue) => queue,
            Err(error) => {
                return HttpResponse::json(400, json!({ "error": error }).to_string());
            }
        };

    let body = serde_json::to_string_pretty(&queue_controls_view(&queue))
        .expect("serialize queue controls view");
    HttpResponse::json(200, body)
}

pub fn queue_submission_response(surface: &SetupSurface, request: HttpRequest) -> HttpResponse {
    let Ok(payload) = serde_json::from_slice::<QueueSubmissionRequest>(&request.body) else {
        return HttpResponse::json(
            400,
            json!({ "error": "queue submission payload must be valid JSON" }).to_string(),
        );
    };

    if payload.message.trim().is_empty() {
        return HttpResponse::json(
            400,
            json!({ "error": "queue message is required" }).to_string(),
        );
    }

    let session_id = payload.session_id.clone();
    let response = match submit_session_queue_control(surface, session_id.as_deref(), payload) {
        Ok(response) => response,
        Err(error) => {
            return HttpResponse::json(400, json!({ "error": error }).to_string());
        }
    };
    let body = serde_json::to_string_pretty(&response).expect("serialize queue submission result");
    HttpResponse::json(200, body)
}

fn request_session_id(path: &str) -> Option<String> {
    let query = path.split_once('?')?.1;
    for pair in query.split('&') {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        if name == "session_id" {
            return Some(percent_decode(value));
        }
    }

    None
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = String::with_capacity(value.len());
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                decoded.push(' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                let hex = &value[index + 1..index + 3];
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    decoded.push(byte as char);
                    index += 3;
                } else {
                    decoded.push('%');
                    index += 1;
                }
            }
            byte => {
                decoded.push(byte as char);
                index += 1;
            }
        }
    }

    decoded
}

fn submit_session_queue_control(
    surface: &SetupSurface,
    session_id: Option<&str>,
    payload: QueueSubmissionRequest,
) -> Result<QueueSubmissionResult, String> {
    let (resolved_session_id, mut session) =
        load_or_create_session_for_request(surface.home(), session_id)?;
    let response = submit_queue_control(
        session.queue_mut(),
        QueueSubmissionRequest {
            session_id: Some(resolved_session_id.clone()),
            ..payload
        },
    );
    persist_session_for_id(surface.home(), &resolved_session_id, &session)?;
    Ok(response)
}
