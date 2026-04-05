use async_trait::async_trait;

use crate::config::SandboxKind;
use crate::error::SandboxError;

#[derive(Debug, Clone)]
pub struct CodeRequest {
    pub code: String,
    pub language: String,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct CommandRequest {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct SandboxResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
    pub backend: SandboxKind,
}

impl SandboxResult {
    pub fn success(stdout: impl Into<String>, backend: SandboxKind) -> Self {
        Self {
            stdout: stdout.into(),
            stderr: String::new(),
            exit_code: 0,
            timed_out: false,
            backend,
        }
    }

    pub fn failure(code: i32, stderr: impl Into<String>, backend: SandboxKind) -> Self {
        Self {
            stdout: String::new(),
            stderr: stderr.into(),
            exit_code: code,
            timed_out: false,
            backend,
        }
    }

    pub fn timeout(secs: u64, backend: SandboxKind) -> Self {
        Self {
            stdout: String::new(),
            stderr: format!("execution timed out after {secs}s"),
            exit_code: -1,
            timed_out: true,
            backend,
        }
    }
}

#[async_trait]
pub trait SandboxRuntime: Send + Sync {
    fn kind(&self) -> SandboxKind;
    fn is_available(&self) -> bool;
    async fn execute_code(&self, request: CodeRequest) -> Result<SandboxResult, SandboxError>;
    async fn execute_command(&self, request: CommandRequest)
        -> Result<SandboxResult, SandboxError>;
    fn supported_languages(&self) -> Vec<String>;
}
