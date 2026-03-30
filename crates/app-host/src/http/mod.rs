use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub mod agent_api;
pub mod agents_api;
pub mod execution_api;
pub mod gateway_api;
pub mod mcp_api;
pub mod openclaw_api;
pub mod queue_api;
pub mod routes;
pub mod setup_api;
pub mod skills_api;
pub mod workspace_api;

use crate::ui_assets::{UiAssetKind, UiAssetLayout};
use matrixclaw_manifests::config::AppConfig;
use matrixclaw_session_runtime::queue::SessionQueue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub body: Vec<u8>,
}

impl HttpRequest {
    pub fn get(path: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::Get,
            path: path.into(),
            body: Vec::new(),
        }
    }

    pub fn post(path: impl Into<String>, body: impl Into<Vec<u8>>) -> Self {
        Self {
            method: HttpMethod::Post,
            path: path.into(),
            body: body.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status_code: u16,
    pub content_type: String,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn new(
        status_code: u16,
        content_type: impl Into<String>,
        body: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            status_code,
            content_type: content_type.into(),
            body: body.into(),
        }
    }

    pub fn text(status_code: u16, body: impl Into<String>) -> Self {
        Self::new(
            status_code,
            "text/plain; charset=utf-8",
            body.into().into_bytes(),
        )
    }

    pub fn html(status_code: u16, body: impl Into<Vec<u8>>) -> Self {
        Self::new(status_code, "text/html; charset=utf-8", body)
    }

    pub fn json(status_code: u16, body: impl Into<String>) -> Self {
        Self::new(status_code, "application/json", body.into().into_bytes())
    }

    pub fn body_text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

#[derive(Debug, Clone)]
pub struct SetupSurface {
    home: PathBuf,
    base_url: String,
    asset_layout: UiAssetLayout,
    contract: routes::SetupServerContract,
    agent_name: String,
    queue: Arc<Mutex<SessionQueue>>,
}

impl SetupSurface {
    pub fn new(home: impl AsRef<Path>, asset_layout: UiAssetLayout) -> Self {
        Self::with_state(home, asset_layout, "default", SessionQueue::default())
    }

    pub fn with_state(
        home: impl AsRef<Path>,
        asset_layout: UiAssetLayout,
        agent_name: impl Into<String>,
        queue: SessionQueue,
    ) -> Self {
        Self {
            home: home.as_ref().to_path_buf(),
            base_url: routes::LOOPBACK_BASE_URL.to_string(),
            asset_layout,
            contract: routes::setup_server_contract(),
            agent_name: agent_name.into(),
            queue: Arc::new(Mutex::new(queue)),
        }
    }

    pub fn contract(&self) -> routes::SetupServerContract {
        self.contract
    }

    pub fn setup_url(&self) -> String {
        format!("{}{}", self.base_url, self.contract.shell_route)
    }

    pub fn handle(&self, request: HttpRequest) -> HttpResponse {
        if routes::is_health_route(&request.path) && request.method == HttpMethod::Get {
            return setup_api::health_response(self);
        }

        if routes::is_submit_route(&request.path) {
            return setup_api::handle_submission(self, request);
        }

        if workspace_api::is_workspace_files_route(&request.path)
            && request.method == HttpMethod::Get
        {
            return workspace_api::list_entries_response(self);
        }

        if workspace_api::is_workspace_reference_route(&request.path)
            && request.method == HttpMethod::Post
        {
            return workspace_api::reference_response(self, request);
        }

        if queue_api::is_queue_state_route(&request.path) && request.method == HttpMethod::Get {
            return queue_api::queue_state_response(self, &request);
        }

        if queue_api::is_queue_submit_route(&request.path) && request.method == HttpMethod::Post {
            return queue_api::queue_submission_response(self, request);
        }

        if execution_api::is_execution_visibility_route(&request.path)
            && request.method == HttpMethod::Get
        {
            return execution_api::execution_visibility_response();
        }

        if execution_api::is_execution_node_route(&request.path)
            && request.method == HttpMethod::Post
        {
            return execution_api::execution_node_response(&request.body);
        }

        if agent_api::is_agent_run_route(&request.path) && request.method == HttpMethod::Post {
            return agent_api::agent_run_response(self, request);
        }

        if openclaw_api::is_openclaw_chat_route(&request.path) && request.method == HttpMethod::Post
        {
            return openclaw_api::openclaw_chat_response(self, request);
        }

        if agents_api::is_agents_directory_route(&request.path) && request.method == HttpMethod::Get
        {
            return agents_api::agents_directory_response(self);
        }

        if agents_api::is_agent_detail_route(&request.path) && request.method == HttpMethod::Get {
            return agents_api::agent_detail_response(self, &request.path);
        }

        if skills_api::is_skills_inventory_route(&request.path) && request.method == HttpMethod::Get
        {
            return skills_api::skills_inventory_response(self, &request.path);
        }

        if skills_api::is_skills_catalog_route(&request.path) && request.method == HttpMethod::Get {
            return skills_api::skills_catalog_response(self);
        }

        if skills_api::is_skills_toggle_route(&request.path) && request.method == HttpMethod::Post {
            return skills_api::toggle_skill_response(self, request);
        }

        if mcp_api::is_mcp_catalog_route(&request.path) && request.method == HttpMethod::Get {
            return mcp_api::mcp_catalog_response(self);
        }

        if gateway_api::is_gateway_catalog_route(&request.path) && request.method == HttpMethod::Get
        {
            return gateway_api::gateway_catalog_response(self);
        }

        if request.method == HttpMethod::Get && routes::is_shell_route(&request.path) {
            if let Some(resolved) = self.asset_layout.resolve_request_path(&request.path) {
                match fs::read(&resolved.file_path) {
                    Ok(body) => {
                        let content_type =
                            content_type_for_asset(resolved.kind, resolved.file_path.as_path());
                        return HttpResponse::new(200, content_type, body);
                    }
                    Err(error) => {
                        return HttpResponse::text(
                            500,
                            format!("failed to read UI asset: {error}"),
                        );
                    }
                }
            }
            return HttpResponse::text(404, "setup shell asset not found");
        }

        HttpResponse::text(404, "route not found")
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn config_ready(&self) -> bool {
        crate::setup::config_path(&self.home).exists()
    }

    pub fn current_agent_name(&self) -> String {
        if let Ok(config) = self.app_config() {
            return config.workspace.name;
        }

        self.agent_name.clone()
    }

    pub fn app_config(&self) -> io::Result<AppConfig> {
        AppConfig::load_from_home(&self.home)
    }

    pub fn workspace_root(&self) -> io::Result<PathBuf> {
        if let Ok(config) = self.app_config() {
            return Ok(config.workspace.root);
        }

        let legacy_root = crate::paths::config_dir(&self.home).join("workspace");
        if legacy_root.exists() {
            return Ok(legacy_root);
        }

        Ok(self.home.join("workspace"))
    }

    pub fn queue(&self) -> Arc<Mutex<SessionQueue>> {
        Arc::clone(&self.queue)
    }
}

pub fn setup_surface_for_home(home: impl AsRef<Path>) -> io::Result<SetupSurface> {
    let home = home.as_ref().to_path_buf();
    Ok(SetupSurface::new(
        &home,
        UiAssetLayout::discover_for_home(&home),
    ))
}

fn content_type_for_asset(kind: UiAssetKind, path: &Path) -> &'static str {
    if kind == UiAssetKind::Shell {
        return "text/html; charset=utf-8";
    }

    match path.extension().and_then(|value| value.to_str()) {
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}
