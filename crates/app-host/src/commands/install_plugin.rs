use std::io;
use std::path::Path;

use crate::compat_registry;
use crate::plugin_launcher::{self, PluginLaunchRequest};
use zstar_manifests::plugin_manifest::{
    install_plugin_package, PluginInstallDiagnostic, PluginInstallOutcome,
};

pub fn install_plugin(
    source_root: impl AsRef<Path>,
    runtime_home: impl AsRef<Path>,
) -> io::Result<PluginInstallOutcome> {
    let source_root_path = source_root.as_ref().to_path_buf();
    let outcome = install_plugin_package(&source_root_path, runtime_home.as_ref())?;
    let _result_message = plugin_install_result_message(&outcome);

    if let PluginInstallOutcome::Imported {
        manifest_path,
        installed_root,
        adapter_path,
        ..
    } = &outcome
    {
        let request = PluginLaunchRequest {
            adapter_path: adapter_path.clone(),
            manifest_path: manifest_path.clone(),
            installed_root: installed_root.clone(),
        };
        let _launch = plugin_launcher::launch_plugin_via_adapter(&request)?;
        let _ = compat_registry::record_plugin_install(runtime_home, &source_root_path, &outcome)?;
    }

    Ok(outcome)
}

pub fn plugin_install_diagnostic(
    outcome: &PluginInstallOutcome,
) -> Option<PluginInstallDiagnostic> {
    outcome.rejection_diagnostic()
}

pub fn plugin_install_result_message(outcome: &PluginInstallOutcome) -> String {
    match outcome {
        PluginInstallOutcome::Imported { installed_root, .. } => {
            format!("plugin installed at {}", installed_root.display())
        }
        PluginInstallOutcome::Rejected { reason } => match outcome.rejection_diagnostic() {
            Some(diagnostic) => format!(
                "plugin install rejected ({:?}): {}",
                diagnostic.code, diagnostic.message
            ),
            None => format!("plugin install rejected: {reason}"),
        },
    }
}
