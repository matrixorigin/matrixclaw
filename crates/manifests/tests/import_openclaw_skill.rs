use std::fs;

use tempfile::tempdir;

use matrixclaw_manifests::skill_manifest::{
    detect_skill_root, import_skill_package, normalize_skill_manifest, SkillInstallOutcome,
    NORMALIZED_MANIFEST_NAME,
};

#[test]
fn import_openclaw_skill() {
    let source = tempdir().expect("source tempdir");
    let runtime_home = tempdir().expect("runtime tempdir");
    fs::write(
        source.path().join("SKILL.md"),
        "# OpenClaw Skill\n\nImported skill package.\n",
    )
    .expect("write skill file");

    let detection = detect_skill_root(source.path())
        .expect("detect skill root")
        .expect("expected SKILL.md to be detected");
    assert_eq!(detection.origin, "openclaw");
    assert_eq!(
        detection.name,
        source.path().file_name().unwrap().to_str().unwrap()
    );

    let normalized = normalize_skill_manifest(&detection);
    assert_eq!(normalized["schemaVersion"], "1");
    assert_eq!(normalized["entry"], "SKILL.md");
    assert_eq!(normalized["compat"]["artifactClass"], "skill_text");

    let outcome = import_skill_package(source.path(), runtime_home.path())
        .expect("import attempt should be classified");

    match outcome {
        SkillInstallOutcome::Imported {
            manifest_path,
            installed_root,
            provenance_path,
        } => {
            assert!(
                manifest_path.ends_with(NORMALIZED_MANIFEST_NAME),
                "expected normalized manifest path"
            );
            assert!(
                installed_root.ends_with(detection.name),
                "expected installed root to use the skill name"
            );
            assert!(
                manifest_path.exists(),
                "expected normalized manifest to be written"
            );
            assert!(
                installed_root.join("SKILL.md").exists(),
                "expected imported skill file to be copied"
            );
            assert!(
                provenance_path.exists(),
                "expected provenance record to be written"
            );
        }
        SkillInstallOutcome::Rejected { reason } => {
            panic!("expected imported compatibility skill, got rejected: {reason}");
        }
    }
}
