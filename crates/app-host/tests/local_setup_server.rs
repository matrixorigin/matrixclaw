use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use zstar_app_host::http::{HttpRequest, SetupSurface};
use zstar_app_host::setup::{config_path, local_setup_server_contract};
use zstar_app_host::ui_assets::{UiAssetLayout, UI_ENTRY_HTML, UI_WORKSPACE_DIR};

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!("zstar-home-{}-{}", std::process::id(), nanos));
    fs::create_dir_all(&home).expect("create temp home");
    home
}

fn temp_repo_root() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let root = env::temp_dir().join(format!("zstar-ui-setup-{}-{}", std::process::id(), nanos));
    fs::create_dir_all(root.join(UI_WORKSPACE_DIR).join("build").join("_app"))
        .expect("create fixture build tree");
    fs::write(
        root.join(UI_WORKSPACE_DIR)
            .join("build")
            .join(UI_ENTRY_HTML),
        "<html><body>setup shell</body></html>",
    )
    .expect("write setup shell");
    root
}

#[test]
fn local_setup_server() {
    let home = temp_home();
    let contract = local_setup_server_contract();

    assert_eq!(
        contract.shell_route, "/setup",
        "first launch should expose a setup shell route"
    );
    assert_eq!(
        contract.submit_route, "/api/setup/config",
        "setup submissions should have a dedicated endpoint"
    );
    assert_eq!(
        contract.health_route, "/healthz",
        "a loopback health route should exist for setup mode"
    );
    assert!(
        contract.browser_first,
        "setup mode should be browser-first when no config exists"
    );

    let surface = SetupSurface::new(&home, UiAssetLayout::from_repo_root(temp_repo_root()));

    let health = surface.handle(HttpRequest::get(contract.health_route));
    assert_eq!(
        health.status_code, 200,
        "health route should be available in setup mode"
    );
    assert!(
        health.body_text().contains("\"mode\":\"setup\""),
        "health response should identify setup mode"
    );

    let shell = surface.handle(HttpRequest::get(contract.shell_route));
    assert_eq!(shell.status_code, 200, "setup route should serve the shell");
    assert!(
        shell.body_text().contains("setup shell"),
        "setup shell should be served through the in-process surface"
    );

    let submit = surface.handle(HttpRequest::post(
        contract.submit_route,
        br#"{"provider":{}}"#.to_vec(),
    ));
    assert_eq!(
        submit.status_code, 400,
        "setup submit route should reject incomplete payloads with structured validation"
    );
    assert!(
        submit.body_text().contains("missing required fields"),
        "validation response should explain the missing setup fields"
    );

    let output = Command::new(env!("CARGO_BIN_EXE_zstar"))
        .env("HOME", &home)
        .output()
        .expect("run zstar startup");

    assert!(
        output.status.success(),
        "startup should remain functional while exposing setup mode: stdout: {}, stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("/setup"),
        "startup should announce the setup surface location: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    assert!(
        !config_path(&home).exists(),
        "first launch should not be treated as complete until setup submission persists config"
    );
}
