use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::http::{HttpRequest, HttpResponse, SetupSurface};
use crate::paths;
use zstar_manifests::config::AppConfig;

pub const WORKSPACE_FILES_ROUTE: &str = "/api/workspace/files";
pub const WORKSPACE_REFERENCE_ROUTE: &str = "/api/workspace/reference";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceEntryKind {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceEntry {
    pub relative_path: PathBuf,
    pub kind: WorkspaceEntryKind,
    pub reference_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceExplorerContract {
    pub files_route: &'static str,
    pub reference_route: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceReferenceRequest {
    pub relative_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceReferenceResponse {
    pub relative_path: PathBuf,
    pub reference_token: String,
}

pub fn workspace_explorer_contract() -> WorkspaceExplorerContract {
    WorkspaceExplorerContract {
        files_route: WORKSPACE_FILES_ROUTE,
        reference_route: WORKSPACE_REFERENCE_ROUTE,
    }
}

pub fn is_workspace_files_route(path: &str) -> bool {
    crate::http::routes::normalize_path(path) == WORKSPACE_FILES_ROUTE
}

pub fn is_workspace_reference_route(path: &str) -> bool {
    crate::http::routes::normalize_path(path) == WORKSPACE_REFERENCE_ROUTE
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceExplorerSurface {
    workspace_root: PathBuf,
}

impl WorkspaceExplorerSurface {
    pub fn new(workspace_root: impl AsRef<Path>) -> Self {
        Self {
            workspace_root: workspace_root.as_ref().to_path_buf(),
        }
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    pub fn list_entries(&self) -> io::Result<Vec<WorkspaceEntry>> {
        let mut entries = Vec::new();
        collect_entries(&self.workspace_root, &self.workspace_root, &mut entries)?;

        entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        Ok(entries)
    }

    pub fn reference_token_for_path(&self, path: impl AsRef<Path>) -> String {
        let normalized = normalize_workspace_path(path.as_ref());
        format!("[[workspace:{normalized}]]")
    }
}

pub fn workspace_surface_for_home(home: impl AsRef<Path>) -> io::Result<WorkspaceExplorerSurface> {
    let home = home.as_ref();
    let legacy_root = paths::config_dir(home).join("workspace");
    let workspace_root = AppConfig::load_from_home(home)
        .map(|config| config.workspace.root)
        .unwrap_or_else(|_| {
            if legacy_root.exists() {
                legacy_root
            } else {
                home.join("workspace")
            }
        });
    Ok(WorkspaceExplorerSurface::new(workspace_root))
}

pub fn list_entries_response(surface: &SetupSurface) -> HttpResponse {
    match WorkspaceExplorerSurface::new(match surface.workspace_root() {
        Ok(root) => root,
        Err(error) => {
            return HttpResponse::text(500, format!("failed to resolve workspace root: {error}"))
        }
    })
    .list_entries()
    {
        Ok(entries) => {
            let body = serde_json::to_string_pretty(&entries).expect("serialize workspace entries");
            HttpResponse::json(200, body)
        }
        Err(error) => HttpResponse::text(500, format!("failed to list workspace entries: {error}")),
    }
}

pub fn reference_response(surface: &SetupSurface, request: HttpRequest) -> HttpResponse {
    let Ok(payload) = serde_json::from_slice::<WorkspaceReferenceRequest>(&request.body) else {
        return HttpResponse::json(
            400,
            json!({
                "error": "workspace reference payload must be valid JSON"
            })
            .to_string(),
        );
    };

    let explorer = match surface.workspace_root() {
        Ok(root) => WorkspaceExplorerSurface::new(root),
        Err(error) => {
            return HttpResponse::json(
                500,
                json!({ "error": format!("failed to resolve workspace root: {error}") })
                    .to_string(),
            )
        }
    };

    let response = WorkspaceReferenceResponse {
        relative_path: payload.relative_path.clone(),
        reference_token: explorer.reference_token_for_path(&payload.relative_path),
    };

    let body = serde_json::to_string_pretty(&response).expect("serialize workspace reference");
    HttpResponse::json(200, body)
}

fn normalize_workspace_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn collect_entries(
    workspace_root: &Path,
    current_dir: &Path,
    entries: &mut Vec<WorkspaceEntry>,
) -> io::Result<()> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path
            .strip_prefix(workspace_root)
            .expect("workspace entries should stay under the workspace root")
            .to_path_buf();
        let kind = if path.is_dir() {
            WorkspaceEntryKind::Directory
        } else {
            WorkspaceEntryKind::File
        };
        let reference_token = format!("[[workspace:{}]]", normalize_workspace_path(&relative_path));

        entries.push(WorkspaceEntry {
            relative_path,
            kind,
            reference_token,
        });

        if path.is_dir() {
            collect_entries(workspace_root, &path, entries)?;
        }
    }

    Ok(())
}
