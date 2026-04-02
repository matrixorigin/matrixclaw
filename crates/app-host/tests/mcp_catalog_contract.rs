use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_app_host::agent_store::{agent_profile_path, AgentProfile};
use matrixclaw_app_host::http::mcp_api::mcp_catalog_path;
use matrixclaw_app_host::http::{HttpRequest, SetupSurface};
use matrixclaw_app_host::session_binding_store::session_bindings_path;
use matrixclaw_app_host::ui_assets::UiAssetLayout;
use serde_json::json;
use serde_json::Value;

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
fn mcp_catalog_contract_uses_file_backed_snapshot_or_defaults() {
    let home = temp_home();
    let profile = AgentProfile {
        agent_name: "atlas".to_string(),
        title: "Atlas".to_string(),
        crown_job: "Research topics.".to_string(),
        memory_summary: "Research context.".to_string(),
        memory_signal_count: 3,
        pinned_memory_count: 1,
        enabled_skills: vec![],
        enabled_mcp_servers: vec!["search-99".to_string()],
        enabled_gateways: vec![],
    };
    let profile_path = agent_profile_path(&home, &profile.agent_name);
    fs::create_dir_all(profile_path.parent().expect("profile parent")).expect("create profile dir");
    fs::write(
        &profile_path,
        serde_json::to_string_pretty(&profile).expect("serialize profile"),
    )
    .expect("write profile");

    let snapshot_path = mcp_catalog_path(&home);
    fs::create_dir_all(snapshot_path.parent().expect("snapshot parent"))
        .expect("create snapshot dir");
    fs::write(
        &snapshot_path,
        json!([
            {
                "name": "search-99",
                "health": "degraded",
                "enabled_by_agent_count": 0
            }
        ])
        .to_string(),
    )
    .expect("write mcp snapshot");

    let bindings_path = session_bindings_path(&home);
    fs::create_dir_all(bindings_path.parent().expect("bindings parent"))
        .expect("create bindings dir");
    fs::write(&bindings_path, "{not-json").expect("write malformed bindings");

    let surface = SetupSurface::new(&home, UiAssetLayout::discover());

    let response = surface.handle(HttpRequest::get("/api/mcp"));
    assert_eq!(response.status_code, 200, "mcp catalog should be exposed");

    let catalog: Value = serde_json::from_slice(&response.body).expect("mcp catalog JSON");
    let records = catalog.as_array().expect("mcp catalog should be an array");
    let search = records
        .iter()
        .find(|record| record.get("name").and_then(Value::as_str) == Some("search-99"))
        .expect("snapshot-backed mcp record should be present");
    assert_eq!(
        search.get("health").and_then(Value::as_str),
        Some("degraded"),
        "mcp catalog should prefer file-backed snapshot health"
    );
    assert_eq!(
        search.get("enabled_by_agent_count").and_then(Value::as_u64),
        Some(1),
        "enabled-by count should be derived from agent profiles even if session bindings are malformed"
    );
}
