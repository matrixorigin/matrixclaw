use serde_json::json;

use crate::agent_store::list_agent_profiles;
use crate::http::{HttpResponse, SetupSurface};

pub const AGENTS_DIRECTORY_ROUTE: &str = "/api/agents";
pub const AGENT_DETAIL_ROUTE: &str = "/api/agents/detail";

pub fn is_agents_directory_route(path: &str) -> bool {
    crate::http::routes::normalize_path(path) == AGENTS_DIRECTORY_ROUTE
}

pub fn is_agent_detail_route(path: &str) -> bool {
    crate::http::routes::normalize_path(path) == AGENT_DETAIL_ROUTE
}

pub fn agents_directory_response(surface: &SetupSurface) -> HttpResponse {
    match list_agent_profiles(surface.home()) {
        Ok(agents) => {
            let body = serde_json::to_string_pretty(&agents).expect("serialize agent directory");
            HttpResponse::json(200, body)
        }
        Err(error) => HttpResponse::json(
            500,
            json!({ "error": format!("failed to load agents directory: {error}") }).to_string(),
        ),
    }
}

pub fn agent_detail_response(surface: &SetupSurface, request_path: &str) -> HttpResponse {
    let Some(agent_name) = agent_name_from_request(request_path) else {
        return HttpResponse::json(
            400,
            json!({ "error": "agent query parameter is required" }).to_string(),
        );
    };

    match list_agent_profiles(surface.home()) {
        Ok(agents) => match agents
            .into_iter()
            .find(|agent| agent.agent_name == agent_name)
        {
            Some(agent) => HttpResponse::json(
                200,
                serde_json::to_string_pretty(&agent).expect("serialize agent detail"),
            ),
            None => HttpResponse::json(
                404,
                json!({ "error": format!("agent not found: {agent_name}") }).to_string(),
            ),
        },
        Err(error) => HttpResponse::json(
            500,
            json!({ "error": format!("failed to load agent detail: {error}") }).to_string(),
        ),
    }
}

fn agent_name_from_request(request_path: &str) -> Option<String> {
    let (_path, query) = request_path.split_once('?')?;
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=')?;
        if key == "agent" && !value.trim().is_empty() {
            return Some(value.trim().to_string());
        }
    }

    None
}
