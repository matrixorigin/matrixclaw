use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use zstar_app_host::http::{HttpRequest, SetupSurface};
use zstar_app_host::ui_assets::UiAssetLayout;
use zstar_manifests::config::{ExecutionMode, ExecutionSettings};

#[derive(Debug, PartialEq, Eq)]
struct ExecutionNodeObservation {
    policy: ExecutionMode,
    status_code: u16,
    backend: Option<String>,
    stdout: Option<String>,
    stderr: Option<String>,
    error: Option<String>,
}

#[test]
fn execution_node_routing() {
    let home = temp_home();
    let surface = SetupSurface::new(&home, UiAssetLayout::discover());

    let observed = vec![
        observe_execution_node(
            &surface,
            ExecutionMode::Disabled,
            ExecutionSettings::disabled(),
            "deny-routed-at-node",
        ),
        observe_execution_node(
            &surface,
            ExecutionMode::Local,
            ExecutionSettings::local_default(),
            "local-routed-at-node",
        ),
        observe_execution_node(
            &surface,
            ExecutionMode::Sandboxed,
            ExecutionSettings::sandboxed(),
            "sandbox-routed-at-node",
        ),
    ];

    assert_eq!(
        observed,
        vec![
            ExecutionNodeObservation {
                policy: ExecutionMode::Disabled,
                status_code: 403,
                backend: None,
                stdout: None,
                stderr: None,
                error: Some("execution is disabled by policy".to_string()),
            },
            ExecutionNodeObservation {
                policy: ExecutionMode::Local,
                status_code: 200,
                backend: Some("local-command".to_string()),
                stdout: Some("local-routed-at-node".to_string()),
                stderr: Some(String::new()),
                error: None,
            },
            ExecutionNodeObservation {
                policy: ExecutionMode::Sandboxed,
                status_code: 200,
                backend: Some("sandbox".to_string()),
                stdout: Some("sandbox-routed-at-node".to_string()),
                stderr: Some(String::new()),
                error: None,
            },
        ],
        "the Execution Node should choose the backend from policy, report it in the structured result, and reject disabled execution before Gateway-level execution logic"
    );
}

fn observe_execution_node(
    surface: &SetupSurface,
    policy: ExecutionMode,
    settings: ExecutionSettings,
    stdout_label: &str,
) -> ExecutionNodeObservation {
    let body = json!({
        "kind": "execution-node.capability-request",
        "capability": "host.command",
        "policy": settings,
        "request": {
            "command": "echo",
            "args": [stdout_label],
            "cwd": null
        }
    });

    let response = surface.handle(HttpRequest::post("/api/node/execution", body.to_string()));

    let parsed: Value =
        serde_json::from_slice(&response.body).expect("execution node response should be JSON");

    ExecutionNodeObservation {
        policy,
        status_code: response.status_code,
        backend: parsed
            .get("result")
            .and_then(|result| result.get("backend"))
            .and_then(Value::as_str)
            .map(str::to_string),
        stdout: parsed
            .get("result")
            .and_then(|result| result.get("stdout"))
            .and_then(Value::as_str)
            .map(str::to_string),
        stderr: parsed
            .get("result")
            .and_then(|result| result.get("stderr"))
            .and_then(Value::as_str)
            .map(str::to_string),
        error: parsed
            .get("error")
            .and_then(Value::as_str)
            .map(str::to_string),
    }
}

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = std::env::temp_dir().join(format!(
        "zstar-execution-node-routing-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&home).expect("create temp home");
    home
}
