use serde::{Deserialize, Serialize};

use crate::execution::execution_mode_label;
use crate::http::HttpResponse;
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

pub fn execution_visibility_snapshot() -> ExecutionVisibilitySnapshot {
    let local_mode_label = execution_mode_label(&ExecutionMode::Local).to_string();
    let visible_backends = vec![
        "local".to_string(),
        "docker".to_string(),
        "boxlite".to_string(),
    ];
    let sandbox_priority = vec!["docker".to_string(), "boxlite".to_string()];

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
