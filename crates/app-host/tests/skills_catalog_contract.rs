use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_app_host::compat_registry::CompatRegistryEntry;
use matrixclaw_app_host::http::skills_api::{compat_registry_path, enabled_skills_path};
use matrixclaw_app_host::http::{HttpRequest, SetupSurface};
use matrixclaw_app_host::ui_assets::UiAssetLayout;
use serde_json::{json, Value};

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!("matrixclaw-home-{}-{}", std::process::id(), nanos));
    fs::create_dir_all(&home).expect("create temp home");
    home
}

fn seeded_home_with_agents_and_skills() -> PathBuf {
    let home = temp_home();

    let source_root = home.join("skills-source").join("research");
    let installed_root = home.join(".matrixclaw").join("installed").join("research");
    fs::create_dir_all(&source_root).expect("create skill source root");
    fs::create_dir_all(&installed_root).expect("create installed skill root");

    CompatRegistryEntry::from_skill_install(
        "research",
        &source_root,
        &installed_root,
        installed_root.join("matrixclaw.skill.json"),
        installed_root.join("provenance.json"),
    )
    .save_to(compat_registry_path(&home))
    .expect("seed compat registry");

    seed_enabled_skills(&home, "atlas", &["research", "summarize"]);
    seed_enabled_skills(&home, "beta", &["research"]);
    seed_enabled_skills(&home, "gamma", &["summarize"]);

    home
}

fn seed_enabled_skills(home: &PathBuf, agent_name: &str, enabled: &[&str]) {
    let path = enabled_skills_path(home, agent_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create enabled skills directory");
    }

    fs::write(
        &path,
        json!({
            "schemaVersion": "1",
            "agentName": agent_name,
            "enabled": enabled,
        })
        .to_string(),
    )
    .expect("seed enabled skills");
}

#[test]
fn skills_catalog_contract_reports_enabled_by_counts() {
    let home = seeded_home_with_agents_and_skills();
    let surface = SetupSurface::new(&home, UiAssetLayout::discover());

    let response = surface.handle(HttpRequest::get("/api/skills/catalog"));
    assert_eq!(
        response.status_code, 200,
        "skills catalog should be exposed"
    );

    let catalog: Value = serde_json::from_slice(&response.body).expect("skills catalog JSON");
    let records = catalog
        .as_array()
        .expect("skills catalog should be an array");
    let research = records
        .iter()
        .find(|record| record.get("name").and_then(Value::as_str) == Some("research"))
        .expect("research skill should be listed");

    assert_eq!(
        research
            .get("enabled_by_agent_count")
            .and_then(Value::as_u64),
        Some(2),
        "research should be enabled by exactly two agents"
    );
    assert_eq!(
        research
            .get("enabled_by_agents")
            .and_then(Value::as_array)
            .expect("research should list enabled agents")
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>(),
        vec!["atlas", "beta"],
        "enabled-by agent names should be surfaced in the catalog"
    );
}
