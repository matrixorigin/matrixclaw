use serde::{Deserialize, Serialize};
use std::io::ErrorKind;

use crate::execution::execution_mode_label;
use crate::http::HttpResponse;
use crate::node::execution::ExecutionNodeCapabilityRequest;
use matrixclaw_manifests::config::ExecutionMode;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionVisibilitySnapshot {
    pub mode_label: String,
    pub visible_backends: Vec<String>,
    pub sandbox_priority: Vec<String>,
    pub sandbox_failure_message: String,
    pub fallback_policy: String,
}

pub const EXECUTION_VISIBILITY_ROUTE: &str = "/api/execution/visibility";
pub const EXECUTION_NODE_ROUTE: &str = "/api/node/execution";

pub fn is_execution_visibility_route(path: &str) -> bool {
    crate::http::routes::normalize_path(path) == EXECUTION_VISIBILITY_ROUTE
}

pub fn is_execution_node_route(path: &str) -> bool {
    crate::http::routes::normalize_path(path) == EXECUTION_NODE_ROUTE
}

pub fn execution_visibility_snapshot() -> ExecutionVisibilitySnapshot {
    let local_mode_label = execution_mode_label(&ExecutionMode::Local).to_string();
    let visible_backends = vec![
        "docker".to_string(),
        "e2b".to_string(),
        "daytona".to_string(),
        "local".to_string(),
    ];
    let sandbox_priority = vec![
        "docker".to_string(),
        "e2b".to_string(),
        "daytona".to_string(),
        "local".to_string(),
    ];

    ExecutionVisibilitySnapshot {
        mode_label: local_mode_label,
        visible_backends,
        sandbox_priority,
        sandbox_failure_message: "sandbox required but unavailable".to_string(),
        fallback_policy: "require sandbox".to_string(),
    }
}

pub fn execution_visibility_response() -> HttpResponse {
    let snapshot = execution_visibility_snapshot();
    let body = serde_json::to_string_pretty(&snapshot).expect("serialize execution visibility");
    HttpResponse::json(200, body)
}

pub fn execution_node_response(request_body: &[u8]) -> HttpResponse {
    let request: ExecutionNodeCapabilityRequest = match serde_json::from_slice(request_body) {
        Ok(request) => request,
        Err(error) => {
            return HttpResponse::json(
                400,
                serde_json::json!({
                    "error": format!("invalid execution node request: {error}"),
                })
                .to_string(),
            );
        }
    };

    match request.execute() {
        Ok(response) => HttpResponse::json(
            200,
            serde_json::to_string(&response).expect("serialize execution node response"),
        ),
        Err(error) => {
            let status_code = match error.kind() {
                ErrorKind::PermissionDenied => 403,
                _ => 400,
            };
            HttpResponse::json(
                status_code,
                serde_json::json!({
                    "error": error.to_string(),
                })
                .to_string(),
            )
        }
    }
}
