use std::io;
use std::path::{Path, PathBuf};

use matrixclaw_manifests::config::{ExecutionBackendSelection, ExecutionMode, ExecutionSettings};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredExecutionResult {
    pub backend: ExecutionBackendSelection,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl StructuredExecutionResult {
    pub fn new(
        backend: ExecutionBackendSelection,
        exit_code: i32,
        stdout: impl Into<String>,
        stderr: impl Into<String>,
    ) -> Self {
        Self {
            backend,
            exit_code,
            stdout: stdout.into(),
            stderr: stderr.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolExecutionMode {
    Local,
    Sandboxed,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolExecutionBackendKind {
    LocalCommand,
    Sandbox,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolExecutionBackendSelection {
    pub mode: ToolExecutionMode,
    pub kind: ToolExecutionBackendKind,
    pub label: String,
    pub requires_docker: bool,
}

impl ToolExecutionBackendSelection {
    pub fn local_default() -> Self {
        Self {
            mode: ToolExecutionMode::Local,
            kind: ToolExecutionBackendKind::LocalCommand,
            label: "local-command".to_string(),
            requires_docker: false,
        }
    }

    pub fn sandbox_with_label(label: impl Into<String>, requires_docker: bool) -> Self {
        Self {
            mode: ToolExecutionMode::Sandboxed,
            kind: ToolExecutionBackendKind::Sandbox,
            label: label.into(),
            requires_docker,
        }
    }

    pub fn disabled_with_label(label: impl Into<String>, requires_docker: bool) -> Self {
        Self {
            mode: ToolExecutionMode::Disabled,
            kind: ToolExecutionBackendKind::Disabled,
            label: label.into(),
            requires_docker,
        }
    }
}

use crate::local_command::{execute_local_command_with_settings, LocalCommandRequest};
use crate::sandbox_backend::{SandboxBackend, SandboxExecutionRequest};

pub trait ExecutionBackendProbe {
    fn docker_available(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionContractPaths {
    pub execution_dir: PathBuf,
    pub execution_config_path: PathBuf,
}

impl ExecutionContractPaths {
    pub fn new(home: impl AsRef<Path>) -> Self {
        let home = home.as_ref();
        let execution_dir = home.join(".matrixclaw").join("config");
        let execution_config_path = execution_dir.join("execution.json");
        Self {
            execution_dir,
            execution_config_path,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionContract {
    pub settings: ExecutionSettings,
    pub tool_backend: ToolExecutionBackendSelection,
}

impl ExecutionContract {
    pub fn local_default() -> Self {
        Self::from_settings(ExecutionSettings::local_default())
    }

    pub fn from_settings(settings: ExecutionSettings) -> Self {
        let tool_backend = match settings.mode {
            ExecutionMode::Local => ToolExecutionBackendSelection::local_default(),
            ExecutionMode::Sandboxed => ToolExecutionBackendSelection::sandbox_with_label(
                settings.backend.label.clone(),
                settings.backend.requires_docker,
            ),
            ExecutionMode::Disabled => ToolExecutionBackendSelection::disabled_with_label(
                settings.backend.label.clone(),
                settings.backend.requires_docker,
            ),
        };

        Self {
            settings,
            tool_backend,
        }
    }

    pub fn save_to_home(&self, home: impl AsRef<Path>) -> io::Result<PathBuf> {
        self.settings.save_to_home(home)
    }
}

pub fn execution_contract_paths(home: impl AsRef<Path>) -> ExecutionContractPaths {
    ExecutionContractPaths::new(home)
}

pub fn default_execution_contract() -> ExecutionContract {
    ExecutionContract::local_default()
}

pub fn load_execution_contract(home: impl AsRef<Path>) -> io::Result<ExecutionContract> {
    let settings = ExecutionSettings::load_from_home(home)?;
    Ok(ExecutionContract::from_settings(settings))
}

pub fn execution_mode_label(mode: &ExecutionMode) -> &'static str {
    match mode {
        ExecutionMode::Local => "local",
        ExecutionMode::Sandboxed => "sandboxed",
        ExecutionMode::Disabled => "disabled",
    }
}

pub fn backend_selection_from_mode(mode: &ExecutionMode) -> ExecutionBackendSelection {
    match mode {
        ExecutionMode::Local => ExecutionBackendSelection::local_command(),
        ExecutionMode::Sandboxed => ExecutionBackendSelection::sandbox(),
        ExecutionMode::Disabled => ExecutionBackendSelection::disabled(),
    }
}

pub fn route_isolated_command<B: SandboxBackend>(
    settings: &ExecutionSettings,
    sandbox_backend: Option<&mut B>,
    request: &SandboxExecutionRequest,
) -> io::Result<StructuredExecutionResult> {
    match settings.mode {
        ExecutionMode::Local => {
            let mut local_request =
                LocalCommandRequest::new(request.command.clone(), request.args.clone());
            if let Some(cwd) = &request.cwd {
                local_request = local_request.with_cwd(cwd.clone());
            }
            let local_result = execute_local_command_with_settings(settings, &local_request)?;
            Ok(StructuredExecutionResult::new(
                local_result.backend,
                local_result.exit_code,
                local_result.stdout,
                local_result.stderr,
            ))
        }
        ExecutionMode::Sandboxed => {
            let backend = sandbox_backend.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "sandbox backend is required when sandbox mode is enabled",
                )
            })?;
            backend.execute(request)
        }
        ExecutionMode::Disabled => Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "execution is disabled by policy",
        )),
    }
}
