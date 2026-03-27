use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub mod execution_api;
pub mod queue_api;
pub mod routes;
pub mod setup_api;
pub mod skills_api;
pub mod workspace_api;

use crate::ui_assets::{UiAssetKind, UiAssetLayout};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupSurface {
    home: PathBuf,
    base_url: String,
    asset_layout: UiAssetLayout,
    contract: routes::SetupServerContract,
}

impl SetupSurface {
    pub fn new(home: impl AsRef<Path>, asset_layout: UiAssetLayout) -> Self {
        Self {
            home: home.as_ref().to_path_buf(),
            base_url: routes::LOOPBACK_BASE_URL.to_string(),
            asset_layout,
            contract: routes::setup_server_contract(),
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

        if request.method == HttpMethod::Get && routes::is_shell_route(&request.path) {
            if let Some(resolved) = self.asset_layout.resolve_request_path(&request.path) {
                if resolved.kind == UiAssetKind::Shell || resolved.kind == UiAssetKind::Static {
                    match fs::read(&resolved.file_path) {
                        Ok(body) => {
                            let content_type = if resolved.kind == UiAssetKind::Shell {
                                "text/html; charset=utf-8"
                            } else {
                                "application/octet-stream"
                            };
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
            }
            return HttpResponse::text(404, "setup shell asset not found");
        }

        HttpResponse::text(404, "route not found")
    }

    pub fn home(&self) -> &Path {
        &self.home
    }
}

pub fn setup_surface_for_home(home: impl AsRef<Path>) -> io::Result<SetupSurface> {
    Ok(SetupSurface::new(home, UiAssetLayout::discover()))
}
