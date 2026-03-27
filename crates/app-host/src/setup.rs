use std::path::{Path, PathBuf};

use matrixclaw_manifests::config::AppConfig;

use crate::execution;
use crate::paths;

pub fn runtime_home(home: impl AsRef<Path>) -> PathBuf {
    paths::runtime_home(home)
}

pub fn config_path(home: impl AsRef<Path>) -> PathBuf {
    paths::config_path(home)
}

pub fn managed_assets_dir(home: impl AsRef<Path>) -> PathBuf {
    paths::managed_assets_dir(home)
}

pub fn setup_required(home: impl AsRef<Path>) -> bool {
    !config_path(home).exists()
}

pub fn ensure_first_launch() -> std::io::Result<()> {
    let home = paths::home_dir();
    if setup_required(&home) {
        let config = AppConfig::default_first_launch(&home);
        let _ = config.save_to_home(&home)?;
    }
    let execution_paths = execution::execution_contract_paths(&home);
    if !execution_paths.execution_config_path.exists() {
        let contract = execution::default_execution_contract();
        let _ = contract.save_to_home(&home)?;
    }
    Ok(())
}
