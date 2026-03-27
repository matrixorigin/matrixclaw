use std::fs;

use tempfile::tempdir;

use matrixclaw_manifests::plugin_manifest::{
    detect_plugin_root, install_plugin_package, normalize_plugin_manifest, PluginInstallOutcome,
    NORMALIZED_MANIFEST_NAME,
};
use matrixclaw_manifests::skill_manifest::SupportTier;

#[test]
fn install_subprocess_plugin() {
    let source = tempdir().expect("source tempdir");
    let runtime_home = tempdir().expect("runtime tempdir");
    fs::write(
        source.path().join("openclaw.plugin.json"),
        r#"{
  "name": "anthropic",
  "description": "OpenClaw subprocess plugin",
  "kind": "provider",
  "transport": {
    "type": "jsonrpc_stdio"
  },
  "capabilities": {
    "provides": ["provider"]
  }
}"#,
    )
    .expect("write plugin manifest");

    let detection = detect_plugin_root(source.path())
        .expect("detect plugin root")
        .expect("expected openclaw.plugin.json to be detected");

    assert_eq!(detection.origin, "openclaw");
    assert_eq!(detection.tier, SupportTier::Shimmed);
    assert_eq!(detection.transport, "jsonrpc_stdio");

    let normalized = normalize_plugin_manifest(&detection);
    assert_eq!(normalized["schemaVersion"], "1");
    assert_eq!(normalized["compat"]["tier"], "shimmed");
    assert_eq!(
        normalized["entrypoint"]["command"],
        "matrixclaw-plugin-adapter"
    );

    let outcome = install_plugin_package(source.path(), runtime_home.path())
        .expect("install attempt should classify");

    match outcome {
        PluginInstallOutcome::Imported {
            manifest_path,
            installed_root,
            provenance_path,
            adapter_path,
        } => {
            assert!(
                manifest_path.ends_with(NORMALIZED_MANIFEST_NAME),
                "expected normalized plugin manifest path"
            );
            assert!(
                manifest_path.exists(),
                "expected normalized plugin manifest to be written"
            );
            assert!(
                installed_root.join("openclaw.plugin.json").exists(),
                "expected imported plugin manifest to be preserved"
            );
            assert!(
                provenance_path.exists(),
                "expected provenance record to be written"
            );
            assert!(
                adapter_path.exists(),
                "expected adapter launch contract to be installed"
            );
        }
        PluginInstallOutcome::Rejected { reason } => {
            panic!(
                "expected shim-compatible plugin install to produce a manifest and adapter launch contract, got rejected: {reason}"
            );
        }
    }
}
