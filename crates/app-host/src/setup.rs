use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use zstar_manifests::config::SetupWizardSubmission;

use crate::execution;
use crate::http::routes::SetupServerContract;
use crate::http::{self, SetupSurface};
use crate::paths;

#[derive(Debug, Clone)]
pub enum StartupMode {
    Ready,
    Setup(SetupSurface),
}

pub fn runtime_home(home: impl AsRef<Path>) -> PathBuf {
    paths::runtime_home(home)
}

pub fn config_path(home: impl AsRef<Path>) -> PathBuf {
    paths::config_path(home)
}

pub fn managed_assets_dir(home: impl AsRef<Path>) -> PathBuf {
    paths::managed_assets_dir(home)
}

pub fn local_setup_server_contract() -> SetupServerContract {
    crate::http::routes::setup_server_contract()
}

pub fn setup_required(home: impl AsRef<Path>) -> bool {
    !config_path(home).exists()
}

pub fn persist_setup_submission(
    home: impl AsRef<Path>,
    submission: &SetupWizardSubmission,
) -> io::Result<()> {
    submission
        .validate()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;

    let home = home.as_ref();
    let config = submission.to_app_config();
    let config_path = crate::paths::config_path(home);
    let execution_path = zstar_manifests::config::ExecutionSettings::execution_path(home);

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let config_tmp = config_path.with_extension("json.tmp");
    let execution_tmp = execution_path.with_extension("json.tmp");

    fs::write(&config_tmp, config.to_json_string()?)?;
    fs::write(&execution_tmp, submission.execution.to_json_string()?)?;

    if let Err(error) = fs::rename(&config_tmp, &config_path) {
        let _ = fs::remove_file(&config_tmp);
        let _ = fs::remove_file(&execution_tmp);
        return Err(error);
    }

    if let Err(error) = fs::rename(&execution_tmp, &execution_path) {
        let _ = fs::remove_file(&execution_tmp);
        let _ = fs::remove_file(&config_path);
        return Err(error);
    }

    Ok(())
}

pub fn startup_mode_for_home(home: impl AsRef<Path>) -> io::Result<StartupMode> {
    let home = home.as_ref();
    let execution_paths = execution::execution_contract_paths(home);
    if !execution_paths.execution_config_path.exists() {
        let contract = execution::default_execution_contract();
        let _ = contract.save_to_home(home)?;
    }

    if setup_required(home) {
        return Ok(StartupMode::Setup(http::setup_surface_for_home(home)?));
    }

    Ok(StartupMode::Ready)
}

pub fn ensure_first_launch() -> io::Result<StartupMode> {
    let home = paths::home_dir();
    startup_mode_for_home(&home)
}
