use std::process::Command;

use async_trait::async_trait;

use crate::config::SandboxKind;
use crate::error::SandboxError;
use crate::runtime::{CodeRequest, CommandRequest, SandboxResult, SandboxRuntime};

pub struct LocalSandboxBackend {
    timeout_secs: u64,
}

impl LocalSandboxBackend {
    pub fn new() -> Self {
        Self { timeout_secs: 30 }
    }

    fn build_shell_command(&self, code: &str, language: &str) -> Result<String, SandboxError> {
        match language {
            "python" | "py" => Ok(format!("echo '{}' | python3", code.replace('\'', "'\\''"))),
            "rust" | "rs" => Ok(format!(
                "echo '{}' | rustc - -o /tmp/script && /tmp/script",
                code.replace('\'', "'\\''")
            )),
            "javascript" | "js" => Ok(format!("echo '{}' | node", code.replace('\'', "'\\''"))),
            "bash" | "sh" => Ok(code.replace('\'', "'\\''")),
            _ => Err(SandboxError::UnsupportedLanguage(language.to_string())),
        }
    }
}

impl Default for LocalSandboxBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SandboxRuntime for LocalSandboxBackend {
    fn kind(&self) -> SandboxKind {
        SandboxKind::Local
    }

    fn is_available(&self) -> bool {
        true
    }

    async fn execute_code(&self, request: CodeRequest) -> Result<SandboxResult, SandboxError> {
        let shell_cmd = self.build_shell_command(&request.code, &request.language)?;
        let cmd_req = CommandRequest {
            command: "sh".into(),
            args: vec!["-c".into(), shell_cmd],
            cwd: None,
            timeout_secs: request.timeout_secs,
        };
        self.execute_command(cmd_req).await
    }

    async fn execute_command(
        &self,
        request: CommandRequest,
    ) -> Result<SandboxResult, SandboxError> {
        let timeout_secs = request.timeout_secs.unwrap_or(self.timeout_secs);
        let command = request.command;
        let args = request.args;
        let cwd = request.cwd;

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            tokio::task::spawn_blocking(move || {
                let mut cmd = Command::new(&command);
                cmd.args(&args);
                if let Some(dir) = cwd {
                    cmd.current_dir(dir);
                }
                cmd.output()
            }),
        )
        .await
        .map_err(|_| SandboxError::Timeout { secs: timeout_secs })?;

        let output =
            result.map_err(|e| SandboxError::Io(std::io::Error::other(e.to_string())))??;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        if exit_code != 0 {
            return Ok(SandboxResult::failure(
                exit_code,
                stderr,
                SandboxKind::Local,
            ));
        }

        Ok(SandboxResult::success(stdout, SandboxKind::Local))
    }

    fn supported_languages(&self) -> Vec<String> {
        vec![
            "python".into(),
            "py".into(),
            "javascript".into(),
            "js".into(),
            "rust".into(),
            "rs".into(),
            "bash".into(),
            "sh".into(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn echo_hello() {
        let backend = LocalSandboxBackend::new();
        let result = backend
            .execute_command(CommandRequest {
                command: "echo".into(),
                args: vec!["hello".into()],
                cwd: None,
                timeout_secs: Some(5),
            })
            .await
            .unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello"));
        assert_eq!(result.backend, SandboxKind::Local);
    }

    #[tokio::test]
    async fn execute_code_bash() {
        let backend = LocalSandboxBackend::new();
        let result = backend
            .execute_code(CodeRequest {
                code: "echo hello from bash".into(),
                language: "bash".into(),
                timeout_secs: Some(5),
            })
            .await
            .unwrap();
        assert!(result.stdout.contains("hello from bash"));
    }

    #[test]
    fn is_always_available() {
        assert!(LocalSandboxBackend::new().is_available());
    }

    #[test]
    fn supported_languages() {
        let backend = LocalSandboxBackend::new();
        assert!(backend
            .supported_languages()
            .contains(&"python".to_string()));
    }
}
