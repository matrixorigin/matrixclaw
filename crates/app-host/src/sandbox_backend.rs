use std::io;
use std::path::PathBuf;

use matrixclaw_agent_core::tool::StructuredExecutionResult;
use matrixclaw_manifests::config::{ExecutionBackendSelection, ExecutionSettings};

use crate::local_command::{execute_local_command, LocalCommandRequest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxExecutionRequest {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
}

impl SandboxExecutionRequest {
    pub fn new(command: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            command: command.into(),
            args,
            cwd: None,
        }
    }
}

pub trait SandboxBackend {
    fn backend_selection(&self) -> ExecutionBackendSelection;
    fn execute(
        &mut self,
        request: &SandboxExecutionRequest,
    ) -> io::Result<StructuredExecutionResult>;
}

pub fn execute_via_sandbox_backend<B: SandboxBackend>(
    backend: &mut B,
    request: &SandboxExecutionRequest,
) -> io::Result<StructuredExecutionResult> {
    backend.execute(request)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalSandboxBackend {
    selection: ExecutionBackendSelection,
}

impl LocalSandboxBackend {
    pub fn from_settings(settings: &ExecutionSettings) -> Self {
        Self {
            selection: settings.backend.clone(),
        }
    }
}

impl SandboxBackend for LocalSandboxBackend {
    fn backend_selection(&self) -> ExecutionBackendSelection {
        self.selection.clone()
    }

    fn execute(
        &mut self,
        request: &SandboxExecutionRequest,
    ) -> io::Result<StructuredExecutionResult> {
        let mut local_request =
            LocalCommandRequest::new(request.command.clone(), request.args.clone());
        if let Some(cwd) = &request.cwd {
            local_request = local_request.with_cwd(cwd.clone());
        }

        let local_result = execute_local_command(&local_request)?;
        Ok(StructuredExecutionResult::new(
            self.selection.clone(),
            local_result.exit_code,
            local_result.stdout,
            local_result.stderr,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxBackendRoute {
    pub selection: ExecutionBackendSelection,
    pub label: String,
}

impl SandboxBackendRoute {
    pub fn from_selection(selection: ExecutionBackendSelection) -> Self {
        Self {
            label: selection.label.clone(),
            selection,
        }
    }
}
