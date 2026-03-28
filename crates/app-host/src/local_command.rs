use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use matrixclaw_manifests::config::{ExecutionBackendSelection, ExecutionMode, ExecutionSettings};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalCommandRequest {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
}

impl LocalCommandRequest {
    pub fn new(program: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            program: program.into(),
            args,
            cwd: None,
        }
    }

    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalCommandResult {
    pub backend: ExecutionBackendSelection,
    pub mode: ExecutionMode,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalCommandBackend {
    selection: ExecutionBackendSelection,
}

impl LocalCommandBackend {
    pub fn default_local() -> Self {
        Self {
            selection: ExecutionBackendSelection::local_command(),
        }
    }

    pub fn from_settings(settings: &ExecutionSettings) -> Self {
        Self {
            selection: settings.backend.clone(),
        }
    }

    pub fn selection(&self) -> &ExecutionBackendSelection {
        &self.selection
    }

    pub fn execute(&self, request: &LocalCommandRequest) -> io::Result<LocalCommandResult> {
        let mut command = Command::new(&request.program);
        command.args(&request.args);
        if let Some(cwd) = &request.cwd {
            command.current_dir(cwd);
        }

        let output = command.output()?;
        Ok(LocalCommandResult {
            backend: self.selection.clone(),
            mode: ExecutionMode::Local,
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

pub fn default_backend() -> LocalCommandBackend {
    LocalCommandBackend::default_local()
}

pub fn execute_local_command(request: &LocalCommandRequest) -> io::Result<LocalCommandResult> {
    default_backend().execute(request)
}

pub fn execute_local_command_with_settings(
    settings: &ExecutionSettings,
    request: &LocalCommandRequest,
) -> io::Result<LocalCommandResult> {
    LocalCommandBackend::from_settings(settings).execute(request)
}

pub fn local_command_settings() -> ExecutionSettings {
    ExecutionSettings::local_default()
}

pub fn local_command_backend_path(home: impl AsRef<Path>) -> PathBuf {
    let home = home.as_ref();
    home.join(".matrixclaw")
        .join("config")
        .join("execution.json")
}
