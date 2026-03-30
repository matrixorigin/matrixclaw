use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_app_host::http::{HttpRequest, SetupSurface};
use matrixclaw_app_host::ui_assets::UiAssetLayout;
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
fn gateway_catalog_contract_uses_file_backed_snapshot_or_defaults() {
    let home = temp_home();
    let surface = SetupSurface::new(&home, UiAssetLayout::discover());

    let response = surface.handle(HttpRequest::get("/api/gateway"));
    assert_eq!(
        response.status_code, 200,
        "gateway catalog should be exposed"
    );

    let catalog: Value = serde_json::from_slice(&response.body).expect("gateway catalog JSON");
    let records = catalog
        .as_array()
        .expect("gateway catalog should be an array");
    assert!(
        !records.is_empty(),
        "gateway catalog should fall back to seeded defaults"
    );
    assert!(
        records.iter().any(|record| {
            record.get("name").and_then(Value::as_str).is_some()
                && record.get("health").and_then(Value::as_str).is_some()
                && record
                    .get("enabled_by_agent_count")
                    .and_then(Value::as_u64)
                    .is_some()
        }),
        "gateway catalog records should include the expected typed fields"
    );
}
