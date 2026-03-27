use std::io;
use std::path::PathBuf;

use matrixclaw_agent_core::tool::StructuredExecutionResult;
use matrixclaw_manifests::config::ExecutionBackendSelection;

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
