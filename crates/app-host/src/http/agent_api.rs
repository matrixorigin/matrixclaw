use std::env;
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::http::{HttpRequest, HttpResponse, SetupSurface};
use crate::ingress::{
    normalize_browser_request, run_ingress_with_provider, run_ingress_with_provider_stream,
};
use crate::live_runtime::LiveRunEvent;
use crate::openai_compatible::OpenAiCompatibleProvider;
use crate::session_binding_store::{bind_session_to_agent, session_binding_for_session_id};

pub const AGENT_RUN_ROUTE: &str = "/api/agent/run";
pub const AGENT_RUN_STREAM_ROUTE: &str = "/api/agent/run/stream";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRunRequest {
    pub prompt: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub agent_name: Option<String>,
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

    let envelope = match normalize_agent_run_request(surface, &payload) {
        Ok(envelope) => envelope,
        Err(response) => {
            return response;
        }
    };

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime");
    let outcome = match rt.block_on(run_ingress_with_provider(
        surface.home(),
        model.clone(),
        &envelope,
        &mut provider,
    )) {
        Ok(outcome) => outcome,
        Err(error) => {
            return HttpResponse::json(502, json!({ "error": error }).to_string());
        }
    };

    let body = serde_json::to_string_pretty(&AgentRunResponse {
        session_id: outcome.live_run.session_id,
        model,
        streamed_message: outcome.live_run.streamed_message,
        final_message: outcome.live_run.final_message,
        events: outcome.live_run.events,
    })
    .expect("serialize agent run response");
    HttpResponse::json(200, body)
}

pub fn stream_agent_run(
    surface: &SetupSurface,
    body: &[u8],
    on_frame: &mut (dyn FnMut(Vec<u8>) -> io::Result<()> + Send),
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

    let envelope = match normalize_agent_run_request(surface, &payload) {
        Ok(envelope) => envelope,
        Err(response) => {
            return on_frame(sse_frame(&AgentRunStreamFrame::Error {
                error: response.body_text(),
            }));
        }
    };

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime");
    let outcome = match rt.block_on(run_ingress_with_provider_stream(
        surface.home(),
        model.clone(),
        &envelope,
        &mut provider,
        &mut |event| {
            let _ = on_frame(sse_frame(&AgentRunStreamFrame::Event { event }));
        },
    )) {
        Ok(outcome) => outcome,
        Err(error) => {
            return on_frame(sse_frame(&AgentRunStreamFrame::Error { error }));
        }
    };

    on_frame(sse_frame(&AgentRunStreamFrame::Complete {
        session_id: outcome.live_run.session_id,
        model: outcome.live_run.model,
        streamed_message: outcome.live_run.streamed_message,
        final_message: outcome.live_run.final_message,
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

fn normalize_agent_run_request(
    surface: &SetupSurface,
    payload: &AgentRunRequest,
) -> Result<crate::ingress::IngressEnvelope, HttpResponse> {
    let session_id = payload
        .session_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(generate_session_id);
    let agent_name = if let Some(agent_name) = payload
        .agent_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        agent_name.to_owned()
    } else if let Some(existing_binding) =
        session_binding_for_session_id(surface.home(), &session_id).map_err(|error| {
            HttpResponse::json(500, json!({ "error": error.to_string() }).to_string())
        })?
    {
        existing_binding.agent_name
    } else {
        surface.current_agent_name()
    };

    let binding = match bind_session_to_agent(surface.home(), &session_id, &agent_name) {
        Ok(binding) => binding,
        Err(error) if error.kind() == io::ErrorKind::InvalidInput => {
            return Err(HttpResponse::json(
                400,
                json!({ "error": error.to_string() }).to_string(),
            ))
        }
        Err(error) => {
            return Err(HttpResponse::json(
                500,
                json!({ "error": error.to_string() }).to_string(),
            ));
        }
    };

    let mut normalized_request = payload.clone();
    normalized_request.session_id = Some(binding.session_id.clone());
    normalized_request.agent_name = Some(binding.agent_name.clone());

    let mut envelope = match normalize_browser_request(&normalized_request) {
        Ok(envelope) => envelope,
        Err(error) => {
            return Err(HttpResponse::json(
                400,
                json!({ "error": error }).to_string(),
            ))
        }
    };

    envelope.target_agent = Some(binding.agent_name);
    Ok(envelope)
}

fn generate_session_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    format!("session-{}-{}", std::process::id(), nanos)
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::session_binding_store::bind_session_to_agent;
    use crate::ui_assets::UiAssetLayout;

    fn temp_home() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let home =
            env::temp_dir().join(format!("matrixclaw-home-{}-{}", std::process::id(), nanos));
        fs::create_dir_all(&home).expect("create temp home");
        home
    }

    #[test]
    fn normalize_agent_run_request_reuses_existing_session_binding_when_agent_is_omitted() {
        let home = temp_home();
        let surface = SetupSurface::new(&home, UiAssetLayout::discover());
        bind_session_to_agent(&home, "session-a", "atlas").expect("seed binding");

        let envelope = normalize_agent_run_request(
            &surface,
            &AgentRunRequest {
                prompt: "hello".to_string(),
                session_id: Some("session-a".to_string()),
                agent_name: None,
            },
        )
        .expect("normalize request");

        assert_eq!(envelope.conversation.session_id, "session-a");
        assert_eq!(envelope.target_agent.as_deref(), Some("atlas"));
    }
}
