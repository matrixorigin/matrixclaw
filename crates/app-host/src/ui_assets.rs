use std::env;
use std::path::{Path, PathBuf};

use crate::paths;

pub const UI_WORKSPACE_DIR: &str = "ui";
pub const UI_BUILD_DIR: &str = "build";
pub const UI_ENTRY_HTML: &str = "index.html";
pub const UI_SETUP_HTML: &str = "setup.html";
pub const UI_WORKSPACE_HTML: &str = "workspace.html";
pub const UI_SKILLS_HTML: &str = "skills.html";

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
        if let Some(build_dir) = env::var_os("MATRIXCLAW_UI_BUILD_DIR") {
            return Self::from_build_dir(build_dir);
        }

        if let Some(root) = env::var_os("MATRIXCLAW_REPO_ROOT") {
            return Self::from_repo_root(root);
        }

        Self::from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
    }

    pub fn discover_for_home(home: impl AsRef<Path>) -> Self {
        if let Some(build_dir) = env::var_os("MATRIXCLAW_UI_BUILD_DIR") {
            return Self::from_build_dir(build_dir);
        }

        let bundled = Self::bundled_for_home(home);
        if bundled.entry_html().is_file() {
            return bundled;
        }

        Self::discover()
    }

    pub fn from_build_dir(build_dir: impl AsRef<Path>) -> Self {
        let build_dir = build_dir.as_ref().to_path_buf();
        let workspace_dir = build_dir
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| build_dir.clone());
        Self {
            workspace_dir,
            build_dir,
        }
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

    pub fn bundled_for_home(home: impl AsRef<Path>) -> Self {
        Self::from_build_dir(
            paths::managed_assets_dir(home)
                .join(UI_WORKSPACE_DIR)
                .join(UI_BUILD_DIR),
        )
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

    pub fn skills_entry_html(&self) -> PathBuf {
        self.build_dir.join(UI_SKILLS_HTML)
    }

    pub fn shell_document_for_route(&self, route: &str) -> Option<PathBuf> {
        let normalized = normalize_request_path(route);
        let shell = self.entry_html();
        if !shell.is_file() {
            return None;
        }

        match normalized.as_str() {
            "/" => Some(shell),
            "/setup" => {
                let setup = self.setup_entry_html();
                Some(if setup.is_file() {
                    setup
                } else {
                    shell.clone()
                })
            }
            "/workspace" => {
                let workspace = self.workspace_entry_html();
                Some(if workspace.is_file() {
                    workspace
                } else {
                    shell.clone()
                })
            }
            "/skills" => {
                let skills = self.skills_entry_html();
                Some(if skills.is_file() {
                    skills
                } else {
                    shell.clone()
                })
            }
            _ => {
                let route_html = self
                    .build_dir
                    .join(normalized.trim_start_matches('/'))
                    .with_extension("html");
                if route_html.is_file() {
                    return Some(route_html);
                }

                if is_client_route(&normalized) {
                    Some(shell)
                } else {
                    None
                }
            }
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
        is_client_route, UiAssetKind, UiAssetLayout, UI_BUILD_DIR, UI_ENTRY_HTML, UI_SETUP_HTML,
        UI_SKILLS_HTML, UI_WORKSPACE_DIR, UI_WORKSPACE_HTML,
    };
    use crate::paths;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_repo_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("zstar-ui-assets-{}-{}", std::process::id(), nanos));
        fs::create_dir_all(root.join(UI_WORKSPACE_DIR).join(UI_BUILD_DIR).join("_app"))
            .expect("create ui build fixture");
        fs::write(
            root.join(UI_WORKSPACE_DIR)
                .join(UI_BUILD_DIR)
                .join(UI_ENTRY_HTML),
            "<html><body>zstar shell</body></html>",
        )
        .expect("write shell document");
        fs::write(
            root.join(UI_WORKSPACE_DIR)
                .join(UI_BUILD_DIR)
                .join(UI_SETUP_HTML),
            "<html><body>zstar setup</body></html>",
        )
        .expect("write setup document");
        fs::write(
            root.join(UI_WORKSPACE_DIR)
                .join(UI_BUILD_DIR)
                .join(UI_WORKSPACE_HTML),
            "<html><body>zstar workspace</body></html>",
        )
        .expect("write workspace document");
        fs::write(
            root.join(UI_WORKSPACE_DIR)
                .join(UI_BUILD_DIR)
                .join(UI_SKILLS_HTML),
            "<html><body>zstar skills</body></html>",
        )
        .expect("write skills document");
        fs::write(
            root.join(UI_WORKSPACE_DIR)
                .join(UI_BUILD_DIR)
                .join("_app")
                .join("app.js"),
            "console.log('zstar');",
        )
        .expect("write static asset");
        root
    }

    fn temp_home() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let home =
            std::env::temp_dir().join(format!("zstar-ui-home-{}-{}", std::process::id(), nanos));
        fs::create_dir_all(&home).expect("create temp home");
        home
    }

    #[test]
    fn repo_root_contract_matches_ui_workspace_layout() {
        let layout = UiAssetLayout::from_repo_root("/tmp/zstar");
        assert_eq!(
            layout.workspace_dir,
            Path::new("/tmp/zstar").join(UI_WORKSPACE_DIR)
        );
        assert_eq!(
            layout.build_dir,
            Path::new("/tmp/zstar")
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
    fn top_level_routes_resolve_to_route_specific_shells() {
        let layout = UiAssetLayout::from_repo_root(temp_repo_root());

        let setup = layout
            .resolve_request_path("/setup")
            .expect("setup should resolve to a shell document");
        assert_eq!(setup.kind, UiAssetKind::Shell);
        assert_eq!(setup.file_path, layout.build_dir.join(UI_SETUP_HTML));

        let workspace = layout
            .resolve_request_path("/workspace")
            .expect("workspace should resolve to a shell document");
        assert_eq!(workspace.kind, UiAssetKind::Shell);
        assert_eq!(
            workspace.file_path,
            layout.build_dir.join(UI_WORKSPACE_HTML)
        );

        let skills = layout
            .resolve_request_path("/skills")
            .expect("skills should resolve to a shell document");
        assert_eq!(skills.kind, UiAssetKind::Shell);
        assert_eq!(skills.file_path, layout.build_dir.join(UI_SKILLS_HTML));
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
    fn discover_for_home_prefers_bundled_runtime_assets() {
        let home = temp_home();
        let bundled_layout = UiAssetLayout::bundled_for_home(&home);
        fs::create_dir_all(bundled_layout.build_dir.join("_app"))
            .expect("create bundled ui fixture");
        fs::write(
            bundled_layout.build_dir.join(UI_ENTRY_HTML),
            "<html><body>bundled shell</body></html>",
        )
        .expect("write bundled shell");

        let discovered = UiAssetLayout::discover_for_home(&home);

        assert_eq!(
            discovered.build_dir,
            paths::managed_assets_dir(&home)
                .join(UI_WORKSPACE_DIR)
                .join(UI_BUILD_DIR)
        );
        assert_eq!(discovered.entry_html(), bundled_layout.entry_html());
    }

    #[test]
    fn asset_detection_excludes_extensionless_client_routes() {
        assert!(is_client_route("/workspace"));
        assert!(is_client_route("/workspace/chat"));
        assert!(!is_client_route("/_app/app.js"));
        assert!(!is_client_route("/zstar-mark.svg"));
    }
}
