use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::paths;

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

pub fn workspace_explorer_contract() -> WorkspaceExplorerContract {
    WorkspaceExplorerContract {
        files_route: WORKSPACE_FILES_ROUTE,
        reference_route: WORKSPACE_REFERENCE_ROUTE,
    }
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
    let workspace_root = paths::config_dir(home).join("workspace");
    Ok(WorkspaceExplorerSurface::new(workspace_root))
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
