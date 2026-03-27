use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_app_host::commands::install_skill::install_skill;
use matrixclaw_app_host::http::skills_api::{
    enabled_skills_path, set_skill_enabled, skills_inventory_for_agent, EnableSkillChange,
    InstalledSkillRecord,
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

fn temp_skill_source() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let source = env::temp_dir()
        .join(format!(
            "matrixclaw-skill-source-{}-{}",
            std::process::id(),
            nanos
        ))
        .join("research");
    fs::create_dir_all(&source).expect("create temp skill source");
    fs::write(
        source.join("SKILL.md"),
        "# research\n\nA reusable imported skill.\n",
    )
    .expect("write skill entry");
    source
}

#[test]
fn skills_inventory_enablement() {
    let home = temp_home();
    let source = temp_skill_source();

    let skill_source_before =
        fs::read_to_string(source.join("SKILL.md")).expect("read source skill before install");

    let outcome = install_skill(&source, &home).expect("install source skill");
    let installed_root = match outcome {
        matrixclaw_manifests::skill_manifest::SkillInstallOutcome::Imported {
            installed_root,
            ..
        } => installed_root,
        matrixclaw_manifests::skill_manifest::SkillInstallOutcome::Rejected { reason } => {
            panic!("expected skill import to succeed, got rejection: {reason}")
        }
    };

    let enabled_path = enabled_skills_path(&home, "default");
    if let Some(parent) = enabled_path.parent() {
        fs::create_dir_all(parent).expect("create enabled skills directory");
    }
    fs::write(
        &enabled_path,
        serde_json::json!({
            "schemaVersion": "1",
            "agentName": "default",
            "enabled": ["research"]
        })
        .to_string(),
    )
    .expect("write enabled skills metadata");

    let inventory = skills_inventory_for_agent(&home, "default").expect("load skills inventory");

    assert_eq!(
        inventory.installed,
        vec![InstalledSkillRecord {
            name: "research".to_string(),
            source_root: source.clone(),
            installed_root: installed_root.clone(),
            manifest_path: installed_root.join("matrixclaw.skill.json"),
            provenance_path: installed_root.join("provenance.json"),
        }],
        "installed skills should be listed separately from enablement state"
    );
    assert!(
        inventory.enabled.iter().any(|record| {
            record.agent_name == "default" && record.enabled.contains(&"research".to_string())
        }),
        "agent-local enablement should be reported separately from the installed inventory"
    );

    let updated = set_skill_enabled(
        &home,
        &EnableSkillChange {
            agent_name: "default".to_string(),
            skill_name: "research".to_string(),
            enabled: false,
        },
    )
    .expect("disable agent-local skill");
    assert!(
        updated.enabled.is_empty(),
        "toggling enablement should only update the agent-local metadata"
    );

    let enabled_metadata =
        fs::read_to_string(&enabled_path).expect("read enabled metadata after toggle");
    assert!(
        enabled_metadata.contains("\"enabled\": []"),
        "enablement toggle should rewrite only the enabled-skills metadata: {enabled_metadata}"
    );

    let skill_source_after = fs::read_to_string(source.join("SKILL.md"))
        .expect("read source skill after inventory lookup");
    assert_eq!(
        skill_source_after, skill_source_before,
        "inventory lookup should not mutate the imported skill source files"
    );
}
