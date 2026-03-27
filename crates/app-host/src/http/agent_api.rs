use std::env;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::http::{HttpRequest, HttpResponse, SetupSurface};
use crate::live_runtime::{LiveRunRequest, SessionBackedLiveRunService};
use crate::openai_compatible::OpenAiCompatibleProvider;

pub const AGENT_RUN_ROUTE: &str = "/api/agent/run";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRunRequest {
    pub prompt: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRunResponse {
    pub session_id: String,
    pub model: String,
    pub streamed_message: String,
    pub final_message: String,
    pub events: Vec<crate::live_runtime::LiveRunEvent>,
}

pub fn is_agent_run_route(path: &str) -> bool {
    crate::http::routes::normalize_path(path) == AGENT_RUN_ROUTE
}

pub fn agent_run_response(surface: &SetupSurface, request: HttpRequest) -> HttpResponse {
    let Ok(payload) = serde_json::from_slice::<AgentRunRequest>(&request.body) else {
        return HttpResponse::json(
            400,
            json!({ "error": "agent run payload must be valid JSON" }).to_string(),
        );
    };

    if payload.prompt.trim().is_empty() {
        return HttpResponse::json(
            400,
            json!({ "error": "prompt is required" }).to_string(),
        );
    }

    let Ok(api_key) = env::var("OPENROUTER_API_KEY") else {
        return HttpResponse::json(
            500,
            json!({ "error": "OPENROUTER_API_KEY is not set" }).to_string(),
        );
    };

    let model = env::var("MATRIXCLAW_LLM_MODEL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| surface.app_config().ok().map(|config| config.provider.model))
        .unwrap_or_else(|| "moonshotai/kimi-k2.5".to_string());

    let mut provider = match build_provider(&api_key, &model) {
        Ok(provider) => provider,
        Err(error) => {
            return HttpResponse::json(500, json!({ "error": error.0 }).to_string());
        }
    };

    let service = SessionBackedLiveRunService::new(surface.home());
    let outcome = match service.run_with_provider(
        model.clone(),
        LiveRunRequest {
            prompt: payload.prompt,
            session_id: payload.session_id,
        },
        &mut provider,
    ) {
        Ok(outcome) => outcome,
        Err(error) => {
            return HttpResponse::json(502, json!({ "error": error }).to_string());
        }
    };

    let body = serde_json::to_string_pretty(&AgentRunResponse {
        session_id: outcome.session_id,
        model,
        streamed_message: outcome.streamed_message,
        final_message: outcome.final_message,
        events: outcome.events,
    })
    .expect("serialize agent run response");
    HttpResponse::json(200, body)
}

fn build_provider(
    api_key: &str,
    model: &str,
) -> Result<OpenAiCompatibleProvider, matrixclaw_agent_core::provider::ProviderError> {
    if let Ok(base_url) = env::var("MATRIXCLAW_OPENAI_BASE_URL") {
        if !base_url.trim().is_empty() {
            return OpenAiCompatibleProvider::with_base_url(base_url, api_key, model);
        }
    }

    OpenAiCompatibleProvider::for_openrouter(api_key, model)
}
