use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_app_host::execution::{execution_contract_paths, ExecutionBackendProbe};
use matrixclaw_app_host::http::{HttpRequest, SetupSurface};
use matrixclaw_app_host::setup::{config_path, local_setup_server_contract};
use matrixclaw_app_host::ui_assets::UiAssetLayout;
use matrixclaw_manifests::config::{
    AuthSettings, ExecutionSettings, ProviderSettings, SetupWizardSubmission, WorkspaceSettings,
};

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!("matrixclaw-home-{}-{}", std::process::id(), nanos));
    fs::create_dir_all(&home).expect("create temp home");
    home
}

fn temp_repo_root() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let root = env::temp_dir().join(format!(
        "matrixclaw-setup-wizard-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(root.join("ui").join("build")).expect("create ui build fixture");
    fs::write(
        root.join("ui").join("build").join("index.html"),
        "<html><body>setup shell</body></html>",
    )
    .expect("write shell fixture");
    root
}

struct NoDockerProbe;

impl ExecutionBackendProbe for NoDockerProbe {
    fn docker_available(&self) -> bool {
        false
    }
}

#[test]
fn setup_wizard_persists_config() {
    let home = temp_home();
    let repo_root = temp_repo_root();
    let surface = SetupSurface::new(&home, UiAssetLayout::from_repo_root(repo_root));
    let contract = local_setup_server_contract();
    let probe = NoDockerProbe;

    assert!(
        !probe.docker_available(),
        "the fixture should simulate a Docker-free host"
    );

    let submission = SetupWizardSubmission::new(
        ProviderSettings::new("openai-compatible", "gpt-5.4"),
        WorkspaceSettings::new("default", home.join("workspace")),
        AuthSettings::new("captured-test-token"),
        ExecutionSettings::local_default(),
    );

    let response = surface.handle(HttpRequest::post(
        contract.submit_route,
        serde_json::to_vec(&submission).expect("serialize setup payload"),
    ));

    assert_eq!(
        response.status_code, 200,
        "valid setup submissions should complete successfully once persistence succeeds"
    );
    assert!(
        response.body_text().contains("\"accepted\":true"),
        "setup submission response should report acceptance"
    );
    assert!(
        response.body_text().contains("\"configWritten\":true"),
        "setup submission response should confirm config persistence"
    );
    assert!(
        config_path(&home).exists(),
        "valid setup submission should persist app config at {:?}, response: {}",
        config_path(&home),
        response.body_text()
    );
    assert!(
        execution_contract_paths(&home)
            .execution_config_path
            .exists(),
        "valid setup submission should persist execution defaults"
    );
}
