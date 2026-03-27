use std::env;
use std::path::{Path, PathBuf};

pub const UI_WORKSPACE_DIR: &str = "ui";
pub const UI_BUILD_DIR: &str = "build";
pub const UI_ENTRY_HTML: &str = "index.html";
pub const UI_SETUP_HTML: &str = "setup/index.html";
pub const UI_WORKSPACE_HTML: &str = "workspace/index.html";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiAssetKind {
    Shell,
    Static,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiResolvedAsset {
    pub kind: UiAssetKind,
    pub request_path: String,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiAssetLayout {
    pub workspace_dir: PathBuf,
    pub build_dir: PathBuf,
}

impl UiAssetLayout {
    pub fn discover() -> Self {
        if let Some(root) = env::var_os("MATRIXCLAW_REPO_ROOT") {
            return Self::from_repo_root(root);
        }

        Self::from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
    }

    pub fn from_manifest_dir(manifest_dir: impl AsRef<Path>) -> Self {
        let crate_dir = PathBuf::from(manifest_dir.as_ref());
        let repo_root = crate_dir
            .parent()
            .and_then(Path::parent)
            .expect("app-host manifest directory should live under crates/app-host");

        Self::from_repo_root(repo_root)
    }

    pub fn from_repo_root(repo_root: impl AsRef<Path>) -> Self {
        let workspace_dir = repo_root.as_ref().join(UI_WORKSPACE_DIR);
        let build_dir = workspace_dir.join(UI_BUILD_DIR);
        Self {
            workspace_dir,
            build_dir,
        }
    }

    pub fn entry_html(&self) -> PathBuf {
        self.build_dir.join(UI_ENTRY_HTML)
    }

    pub fn setup_entry_html(&self) -> PathBuf {
        self.build_dir.join(UI_SETUP_HTML)
    }

    pub fn workspace_entry_html(&self) -> PathBuf {
        self.build_dir.join(UI_WORKSPACE_HTML)
    }

    pub fn shell_document_for_route(&self, route: &str) -> Option<PathBuf> {
        let normalized = normalize_request_path(route);
        let shell = self.entry_html();
        if !shell.is_file() {
            return None;
        }

        match normalized.as_str() {
            "/" => Some(shell),
            _ if is_client_route(&normalized) => Some(shell),
            _ => None,
        }
    }

    pub fn resolve_request_path(&self, request_path: &str) -> Option<UiResolvedAsset> {
        let normalized = normalize_request_path(request_path);

        if let Some(file_path) = self.static_asset_for_request(&normalized) {
            return Some(UiResolvedAsset {
                kind: UiAssetKind::Static,
                request_path: normalized,
                file_path,
            });
        }

        self.shell_document_for_route(&normalized)
            .map(|file_path| UiResolvedAsset {
                kind: UiAssetKind::Shell,
                request_path: normalized,
                file_path,
            })
    }

    fn static_asset_for_request(&self, request_path: &str) -> Option<PathBuf> {
        let relative = request_path.trim_start_matches('/');
        if relative.is_empty() {
            return None;
        }

        let candidate = self.build_dir.join(relative);
        candidate.is_file().then_some(candidate)
    }
}

fn normalize_request_path(request_path: &str) -> String {
    let trimmed = request_path.trim();
    if trimmed.is_empty() || trimmed == "/" {
        return "/".to_string();
    }

    let without_query = trimmed
        .split_once(['?', '#'])
        .map(|(path, _)| path)
        .unwrap_or(trimmed);

    if without_query.starts_with('/') {
        without_query.to_string()
    } else {
        format!("/{without_query}")
    }
}

fn is_client_route(request_path: &str) -> bool {
    let trimmed = request_path.trim_start_matches('/');
    if trimmed.is_empty() {
        return true;
    }

    let last_segment = trimmed.rsplit('/').next().unwrap_or(trimmed);
    !last_segment.contains('.')
}

#[cfg(test)]
mod tests {
    use super::{
        is_client_route, UiAssetKind, UiAssetLayout, UI_BUILD_DIR, UI_ENTRY_HTML, UI_WORKSPACE_DIR,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_repo_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "matrixclaw-ui-assets-{}-{}",
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(root.join(UI_WORKSPACE_DIR).join(UI_BUILD_DIR).join("_app"))
            .expect("create ui build fixture");
        fs::write(
            root.join(UI_WORKSPACE_DIR)
                .join(UI_BUILD_DIR)
                .join(UI_ENTRY_HTML),
            "<html><body>matrixclaw shell</body></html>",
        )
        .expect("write shell document");
        fs::write(
            root.join(UI_WORKSPACE_DIR)
                .join(UI_BUILD_DIR)
                .join("_app")
                .join("app.js"),
            "console.log('matrixclaw');",
        )
        .expect("write static asset");
        root
    }

    #[test]
    fn repo_root_contract_matches_ui_workspace_layout() {
        let layout = UiAssetLayout::from_repo_root("/tmp/matrixclaw");
        assert_eq!(
            layout.workspace_dir,
            Path::new("/tmp/matrixclaw").join(UI_WORKSPACE_DIR)
        );
        assert_eq!(
            layout.build_dir,
            Path::new("/tmp/matrixclaw")
                .join(UI_WORKSPACE_DIR)
                .join(UI_BUILD_DIR)
        );
    }

    #[test]
    fn client_routes_resolve_to_root_shell() {
        let layout = UiAssetLayout::from_repo_root(temp_repo_root());
        let resolved = layout
            .resolve_request_path("/workspace/chat")
            .expect("client routes should resolve to the shell entry");
        assert_eq!(resolved.kind, UiAssetKind::Shell);
        assert_eq!(resolved.file_path, layout.entry_html());
    }

    #[test]
    fn static_assets_prefer_exact_build_files() {
        let layout = UiAssetLayout::from_repo_root(temp_repo_root());
        let resolved = layout
            .resolve_request_path("/_app/app.js")
            .expect("static assets should resolve exactly");
        assert_eq!(resolved.kind, UiAssetKind::Static);
        assert_eq!(
            resolved.file_path,
            layout.build_dir.join("_app").join("app.js")
        );
    }

    #[test]
    fn asset_detection_excludes_extensionless_client_routes() {
        assert!(is_client_route("/workspace"));
        assert!(is_client_route("/workspace/chat"));
        assert!(!is_client_route("/_app/app.js"));
        assert!(!is_client_route("/matrixclaw-mark.svg"));
    }
}
