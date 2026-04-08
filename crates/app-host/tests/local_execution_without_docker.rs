use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use zstar_app_host::execution::{
    backend_selection_from_mode, default_execution_contract, execution_contract_paths,
    ExecutionBackendProbe,
};
use zstar_manifests::config::{ExecutionMode, ExecutionSettings};

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!("zstar-home-{}-{}", std::process::id(), nanos));
    fs::create_dir_all(&home).expect("create temp home");
    home
}

struct NoDockerProbe;

impl ExecutionBackendProbe for NoDockerProbe {
    fn docker_available(&self) -> bool {
        false
    }
}

#[test]
fn local_execution_without_docker() {
    let home = temp_home();
    let probe = NoDockerProbe;

    assert!(
        !probe.docker_available(),
        "the test fixture must simulate a Docker-free host"
    );

    let output = Command::new(env!("CARGO_BIN_EXE_zstar"))
        .env("HOME", &home)
        .env_remove("DOCKER_HOST")
        .output()
        .expect("run zstar startup");

    assert!(
        output.status.success(),
        "startup should remain functional without Docker: stdout: {}, stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let contract_paths = execution_contract_paths(&home);
    assert!(
        contract_paths.execution_config_path.exists(),
        "expected startup to persist execution mode at {:?}, but it was missing",
        contract_paths.execution_config_path
    );

    let expected_contract = default_execution_contract();
    let expected_mode_label =
        zstar_app_host::execution::execution_mode_label(&expected_contract.settings.mode);
    let expected_backend = backend_selection_from_mode(&ExecutionMode::Local);
    let expected_settings = ExecutionSettings::local_default();
    assert_eq!(
        expected_settings.mode,
        ExecutionMode::Local,
        "default execution mode should be local"
    );
    assert_eq!(
        expected_backend.label, "local-command",
        "local mode should map to the local command backend"
    );
    assert_eq!(
        expected_mode_label, "local",
        "default mode label should be local"
    );
}
