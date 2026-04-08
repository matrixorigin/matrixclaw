use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use zstar_app_host::http::{HttpRequest, SetupSurface};
use zstar_app_host::ui_assets::UiAssetLayout;

#[test]
fn execution_node_contract() {
    let home = temp_home();
    let surface = SetupSurface::new(&home, UiAssetLayout::discover());

    let response = surface.handle(HttpRequest::post(
        "/api/node/execution",
        json!({
            "kind": "execution-node.capability-request",
            "capability": "host.command",
            "request": {
                "command": "echo",
                "args": ["node-contract"],
                "cwd": null
            }
        })
        .to_string(),
    ));

    assert_eq!(
        response.status_code, 200,
        "the runtime should reach execution through the Execution Node boundary"
    );
    assert_eq!(
        response.content_type, "application/json",
        "the Execution Node should return a structured capability result"
    );

    let body: Value =
        serde_json::from_slice(&response.body).expect("Execution Node response should be JSON");

    assert_eq!(
        body["request"]["kind"].as_str(),
        Some("execution-node.capability-request"),
        "the request should be represented as a Node-specific capability request"
    );
    assert_eq!(
        body["request"]["capability"].as_str(),
        Some("host.command"),
        "the node request should preserve the capability being executed"
    );
    assert_eq!(
        body["result"]["kind"].as_str(),
        Some("execution-node.capability-result"),
        "the node should return a structured capability result"
    );
    assert_eq!(
        body["result"]["backend"].as_str(),
        Some("node"),
        "the runtime should not need to know local or sandbox backend implementation details"
    );
    assert_eq!(body["result"]["exit_code"].as_i64(), Some(0));
    assert_eq!(body["result"]["stdout"].as_str(), Some("node-contract"));
    assert_eq!(body["result"]["stderr"].as_str(), Some(""));
}

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = std::env::temp_dir().join(format!(
        "zstar-execution-node-contract-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&home).expect("create temp home");
    home
}
