use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_app_host::setup::config_path;
use matrixclaw_manifests::config::{
    AppConfig, AuthSettings, ManagedAssetsSettings, ProviderSettings, WorkspaceSettings,
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

#[test]
fn first_launch_setup() {
    let home = temp_home();
    let expected_config = config_path(&home);
    let provider = ProviderSettings::new("openai-compatible", "gpt-5.4");
    let workspace = WorkspaceSettings::new("default", home.join("workspace"));
    let auth = AuthSettings::new("captured-test-token");
    let _expected = AppConfig::new(provider, workspace, auth, ManagedAssetsSettings::default());

    let output = Command::new(env!("CARGO_BIN_EXE_matrixclaw"))
        .env("HOME", &home)
        .output()
        .expect("run matrixclaw");

    assert!(
        expected_config.exists(),
        "expected first launch to persist config at {:?}, stdout: {}, stderr: {}",
        expected_config,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
