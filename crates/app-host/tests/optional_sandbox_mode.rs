use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_app_host::execution::{
    execution_contract_paths, load_execution_contract, route_isolated_command,
    ExecutionBackendProbe, StructuredExecutionResult, ToolExecutionBackendKind,
};
use matrixclaw_app_host::sandbox_backend::{
    SandboxBackend, SandboxBackendRoute, SandboxExecutionRequest,
};
use matrixclaw_manifests::config::{ExecutionBackendKind, ExecutionMode, ExecutionSettings};

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!("matrixclaw-home-{}-{}", std::process::id(), nanos));
    fs::create_dir_all(&home).expect("create temp home");
    home
}

struct NoDockerProbe;

impl ExecutionBackendProbe for NoDockerProbe {
    fn docker_available(&self) -> bool {
        false
    }
}

#[derive(Default)]
struct RecordingSandboxBackend {
    calls: usize,
}

impl SandboxBackend for RecordingSandboxBackend {
    fn backend_selection(&self) -> matrixclaw_manifests::config::ExecutionBackendSelection {
        ExecutionSettings::sandboxed().backend
    }

    fn execute(
        &mut self,
        request: &SandboxExecutionRequest,
    ) -> std::io::Result<StructuredExecutionResult> {
        self.calls += 1;
        let route = SandboxBackendRoute::from_selection(self.backend_selection());
        Ok(StructuredExecutionResult::new(
            route.selection,
            0,
            format!("sandboxed: {}", request.command),
            "",
        ))
    }
}

#[test]
fn optional_sandbox_mode() {
    let home = temp_home();
    let probe = NoDockerProbe;
    assert!(
        !probe.docker_available(),
        "the test fixture must simulate Docker-free startup"
    );

    let sandbox_settings = ExecutionSettings::sandboxed();
    let _sandbox_config_path = sandbox_settings
        .save_to_home(&home)
        .expect("write sandbox execution config");

    let mut backend = RecordingSandboxBackend::default();
    let structured = backend
        .execute(&SandboxExecutionRequest::new(
            "sh",
            vec!["-lc".to_string(), "echo isolated".to_string()],
        ))
        .expect("sandbox backend should produce a structured result");
    assert_eq!(
        structured.backend.kind,
        ExecutionBackendKind::Sandbox,
        "sandbox backend double should report the sandbox route"
    );
    assert_eq!(
        backend.calls, 1,
        "sandbox backend double should be invoked once"
    );

    let output = Command::new(env!("CARGO_BIN_EXE_matrixclaw"))
        .env("HOME", &home)
        .env_remove("DOCKER_HOST")
        .output()
        .expect("run matrixclaw startup");

    assert!(
        output.status.success(),
        "startup should remain functional with sandbox mode configured: stdout: {}, stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let contract = load_execution_contract(&home).expect("load persisted execution contract");
    assert_eq!(
        contract.settings.mode,
        ExecutionMode::Sandboxed,
        "sandbox mode should be explicit in the persisted contract"
    );
    assert_eq!(
        contract.tool_backend.kind,
        ToolExecutionBackendKind::Sandbox,
        "sandbox mode should resolve to the sandbox backend kind"
    );

    let contract_paths = execution_contract_paths(&home);
    assert!(
        contract_paths.execution_config_path.exists(),
        "expected execution config to remain available at {:?}",
        contract_paths.execution_config_path
    );

    let routed = route_isolated_command(
        &sandbox_settings,
        Some(&mut backend),
        &SandboxExecutionRequest::new("sh", vec!["-lc".to_string(), "echo isolated".to_string()]),
    )
    .expect("sandbox routing should succeed");
    assert_eq!(
        routed.backend.kind,
        ExecutionBackendKind::Sandbox,
        "sandboxed execution should route through the configured sandbox backend"
    );
    assert_eq!(
        routed.stdout, "sandboxed: sh",
        "structured execution results should preserve backend output"
    );
}
