use std::io;
use std::path::Path;

use crate::compat_registry;
use matrixclaw_manifests::skill_manifest::{import_skill_package, SkillInstallOutcome};

pub fn install_skill(
    source_root: impl AsRef<Path>,
    runtime_home: impl AsRef<Path>,
) -> io::Result<SkillInstallOutcome> {
    let source_root_path = source_root.as_ref().to_path_buf();
    let outcome = import_skill_package(&source_root_path, runtime_home.as_ref())?;
    let _ = compat_registry::record_skill_install(runtime_home, &source_root_path, &outcome)?;
    Ok(outcome)
}
