use std::env;
use std::io;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::http::{HttpRequest, HttpResponse, SetupSurface};
use crate::live_runtime::{LiveRunEvent, LiveRunRequest, SessionBackedLiveRunService};
use crate::openai_compatible::OpenAiCompatibleProvider;

pub const AGENT_RUN_ROUTE: &str = "/api/agent/run";
pub const AGENT_RUN_STREAM_ROUTE: &str = "/api/agent/run/stream";

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
    pub events: Vec<LiveRunEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentRunStreamFrame {
    Event {
        event: LiveRunEvent,
    },
    Complete {
        session_id: String,
        model: String,
        streamed_message: String,
        final_message: String,
    },
    Error {
        error: String,
    },
}

pub fn is_agent_run_route(path: &str) -> bool {
    crate::http::routes::normalize_path(path) == AGENT_RUN_ROUTE
}

pub fn is_agent_run_stream_route(path: &str) -> bool {
    crate::http::routes::normalize_path(path) == AGENT_RUN_STREAM_ROUTE
}

pub fn agent_run_response(surface: &SetupSurface, request: HttpRequest) -> HttpResponse {
    let payload = match parse_agent_run_request(&request.body) {
        Ok(payload) => payload,
        Err(response) => return response,
    };

    let model = resolve_model(surface);
    let mut provider = match build_provider_from_env(surface, &model) {
        Ok(provider) => provider,
        Err(response) => return response,
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

pub fn stream_agent_run(
    surface: &SetupSurface,
    body: &[u8],
    on_frame: &mut dyn FnMut(Vec<u8>) -> io::Result<()>,
) -> io::Result<()> {
    let payload = match parse_agent_run_request(body) {
        Ok(payload) => payload,
        Err(response) => {
            return on_frame(sse_frame(&AgentRunStreamFrame::Error {
                error: response.body_text(),
            }));
        }
    };

    let model = resolve_model(surface);
    let mut provider = match build_provider_from_env(surface, &model) {
        Ok(provider) => provider,
        Err(response) => {
            return on_frame(sse_frame(&AgentRunStreamFrame::Error {
                error: response.body_text(),
            }));
        }
    };

    let service = SessionBackedLiveRunService::new(surface.home());
    let outcome = match service.run_with_provider_and_queue_stream(
        model.clone(),
        LiveRunRequest {
            prompt: payload.prompt,
            session_id: payload.session_id,
        },
        None,
        &mut provider,
        &mut |event| {
            let _ = on_frame(sse_frame(&AgentRunStreamFrame::Event { event }));
        },
    ) {
        Ok(outcome) => outcome,
        Err(error) => {
            return on_frame(sse_frame(&AgentRunStreamFrame::Error { error }));
        }
    };

    on_frame(sse_frame(&AgentRunStreamFrame::Complete {
        session_id: outcome.session_id,
        model: outcome.model,
        streamed_message: outcome.streamed_message,
        final_message: outcome.final_message,
    }))
}

pub fn sse_frame(frame: &AgentRunStreamFrame) -> Vec<u8> {
    let payload = serde_json::to_string(frame).expect("serialize SSE frame");
    format!("data: {payload}\n\n").into_bytes()
}

fn parse_agent_run_request(body: &[u8]) -> Result<AgentRunRequest, HttpResponse> {
    let Ok(payload) = serde_json::from_slice::<AgentRunRequest>(body) else {
        return Err(HttpResponse::json(
            400,
            json!({ "error": "agent run payload must be valid JSON" }).to_string(),
        ));
    };

    if payload.prompt.trim().is_empty() {
        return Err(HttpResponse::json(
            400,
            json!({ "error": "prompt is required" }).to_string(),
        ));
    }

    Ok(payload)
}

pub(crate) fn resolve_model(surface: &SetupSurface) -> String {
    env::var("MATRIXCLAW_LLM_MODEL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            surface
                .app_config()
                .ok()
                .map(|config| config.provider.model)
        })
        .unwrap_or_else(|| "moonshotai/kimi-k2.5".to_string())
}

pub(crate) fn build_provider_from_env(
    surface: &SetupSurface,
    model: &str,
) -> Result<OpenAiCompatibleProvider, HttpResponse> {
    let Ok(api_key) = env::var("OPENROUTER_API_KEY") else {
        return Err(HttpResponse::json(
            500,
            json!({ "error": "OPENROUTER_API_KEY is not set" }).to_string(),
        ));
    };

    let _ = surface;
    build_provider(&api_key, model)
        .map_err(|error| HttpResponse::json(500, json!({ "error": error.0 }).to_string()))
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
