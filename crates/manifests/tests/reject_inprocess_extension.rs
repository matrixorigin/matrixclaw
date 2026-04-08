use std::fs;

use tempfile::tempdir;

use zstar_manifests::plugin_manifest::{
    install_plugin_package, PluginInstallOutcome, PluginInstallReasonCode, PLUGIN_ENTRY_NAME,
};
use zstar_manifests::skill_manifest::SupportTier;

#[test]
fn reject_inprocess_extension() {
    let source = tempdir().expect("source tempdir");
    let runtime_home = tempdir().expect("runtime tempdir");

    fs::write(
        source.path().join(PLUGIN_ENTRY_NAME),
        r#"{
  "name": "bun-bridge-freeform",
  "description": "OpenClaw extension that expects direct Bun and TypeScript runtime internals",
  "kind": "hook",
  "transport": {
    "type": "jsonrpc_stdio"
  },
  "capabilities": {
    "provides": ["hook"]
  }
}"#,
    )
    .expect("write plugin manifest");

    fs::write(
        source.path().join("package.json"),
        r#"{
  "name": "bun-bridge-freeform",
  "type": "module"
}"#,
    )
    .expect("write package manifest");

    fs::write(
        source.path().join("index.ts"),
        "export const plugin = () => 'in-process extension';\n",
    )
    .expect("write TypeScript entrypoint");

    let outcome = install_plugin_package(source.path(), runtime_home.path())
        .expect("install attempt should classify");

    match outcome {
        PluginInstallOutcome::Imported {
            manifest_path,
            installed_root,
            provenance_path,
            adapter_path,
        } => {
            panic!(
                "expected in-process OpenClaw extension to be rejected with a bridge/manual-rewrite diagnostic, but it was silently accepted at {} with adapter {} and provenance {} (installed root {}), tier {:?}",
                manifest_path.display(),
                adapter_path.display(),
                provenance_path.display(),
                installed_root.display(),
                SupportTier::Shimmed
            );
        }
        PluginInstallOutcome::Rejected { reason } => {
            let diagnostic = PluginInstallOutcome::Rejected {
                reason: reason.clone(),
            }
            .rejection_diagnostic()
            .expect("expected a machine-readable rejection diagnostic");
            assert_eq!(
                diagnostic.code,
                PluginInstallReasonCode::InProcessExtension,
                "expected in-process extension rejection code"
            );
            assert!(
                diagnostic.message.contains("bridge")
                    || diagnostic.message.contains("manual")
                    || diagnostic.message.contains("rewrite"),
                "expected bridge/manual-rewrite guidance, got: {}",
                diagnostic.message
            );
        }
    }
}
