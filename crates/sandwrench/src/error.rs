use thiserror::Error;

#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("docker not available: {0}")]
    DockerUnavailable(String),
    #[error("execution failed with exit code {code}: {stderr}")]
    ExecutionFailed { code: i32, stderr: String },
    #[error("execution timed out after {secs}s")]
    Timeout { secs: u64 },
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("backend unavailable: {0}")]
    BackendUnavailable(String),
    #[error("configuration error: {0}")]
    Config(String),
    #[error("http request failed: {0}")]
    HttpError(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
