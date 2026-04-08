use serde::{Deserialize, Serialize};
use serde_json::json;

use zstar_manifests::config::SetupWizardSubmission;

use crate::http::{HttpMethod, HttpRequest, HttpResponse, SetupSurface};
use crate::setup;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupValidationResponse {
    pub accepted: bool,
    pub config_written: bool,
    pub next: Option<String>,
    pub error: Option<String>,
}

pub fn health_response(surface: &SetupSurface) -> HttpResponse {
    HttpResponse::json(
        200,
        json!({
            "mode": "setup",
            "baseUrl": surface.setup_url(),
            "configReady": !setup::setup_required(surface.home())
        })
        .to_string(),
    )
}

pub fn handle_submission(surface: &SetupSurface, request: HttpRequest) -> HttpResponse {
    if request.method != HttpMethod::Post {
        return HttpResponse::json(
            405,
            json!({
                "accepted": false,
                "configWritten": false,
                "error": "setup submissions require POST"
            })
            .to_string(),
        );
    }

    let Ok(body) = serde_json::from_slice::<serde_json::Value>(&request.body) else {
        return HttpResponse::json(
            400,
            json!({
                "accepted": false,
                "configWritten": false,
                "error": "setup payload must be valid JSON"
            })
            .to_string(),
        );
    };

    let required = ["provider", "workspace", "auth", "execution"];
    let missing: Vec<&str> = required
        .into_iter()
        .filter(|field| body.get(field).is_none())
        .collect();

    if !missing.is_empty() {
        return HttpResponse::json(
            400,
            json!({
                "accepted": false,
                "configWritten": false,
                "error": format!("missing required fields: {}", missing.join(", "))
            })
            .to_string(),
        );
    }

    let Ok(submission): Result<SetupWizardSubmission, _> = serde_json::from_value(body) else {
        return HttpResponse::json(
            400,
            json!({
                "accepted": false,
                "configWritten": false,
                "error": "setup payload could not be normalized"
            })
            .to_string(),
        );
    };

    if let Err(error) = submission.validate() {
        return HttpResponse::json(
            400,
            json!(SetupValidationResponse {
                accepted: false,
                config_written: false,
                next: None,
                error: Some(error.to_string()),
            })
            .to_string(),
        );
    }

    if let Err(error) = setup::persist_setup_submission(surface.home(), &submission) {
        return HttpResponse::json(
            500,
            json!(SetupValidationResponse {
                accepted: false,
                config_written: false,
                next: None,
                error: Some(format!("failed to persist setup payload: {error}")),
            })
            .to_string(),
        );
    }

    HttpResponse::json(
        200,
        json!(SetupValidationResponse {
            accepted: true,
            config_written: true,
            next: Some("workspace".to_string()),
            error: None,
        })
        .to_string(),
    )
}
