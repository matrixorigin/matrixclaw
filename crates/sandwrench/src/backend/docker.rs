use std::process::Command;

use async_trait::async_trait;

use crate::config::{SandboxConfig, SandboxKind};
use crate::error::SandboxError;
use crate::runtime::{CodeRequest, CommandRequest, SandboxResult, SandboxRuntime};

pub struct DockerSandboxBackend {
    config: SandboxConfig,
}

impl DockerSandboxBackend {
    pub fn new(config: SandboxConfig) -> Result<Self, SandboxError> {
        let image = config.docker_image.as_deref().unwrap_or("ubuntu:22.04");
        if image.is_empty() {
            return Err(SandboxError::Config("docker_image cannot be empty".into()));
        }
        Ok(Self { config })
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

#[async_trait]
impl SandboxRuntime for DockerSandboxBackend {
    fn kind(&self) -> SandboxKind {
        SandboxKind::Docker
    }

    fn is_available(&self) -> bool {
        Command::new("docker")
            .arg("info")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    async fn execute_code(&self, request: CodeRequest) -> Result<SandboxResult, SandboxError> {
        let shell_cmd = self.build_shell_command(&request.code, &request.language)?;
        let cmd_req = CommandRequest {
            command: shell_cmd,
            args: vec!["sh".into(), "-c".into()],
            cwd: None,
            timeout_secs: request.timeout_secs,
        };
        self.execute_command(cmd_req).await
    }

    async fn execute_command(
        &self,
        request: CommandRequest,
    ) -> Result<SandboxResult, SandboxError> {
        let timeout_secs = request
            .timeout_secs
            .unwrap_or(self.config.default_timeout_secs);
        let image = self
            .config
            .docker_image
            .clone()
            .unwrap_or_else(|| "ubuntu:22.04".into());
        let memory = self.config.memory_limit.clone();
        let cpu = self.config.cpu_limit;
        let network = if self.config.network_enabled {
            "bridge"
        } else {
            "none"
        };
        let command = request.command;
        let args = request.args;

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            tokio::task::spawn_blocking(move || {
                let mut cmd = Command::new("docker");
                cmd.arg("run")
                    .arg("--rm")
                    .arg("--network")
                    .arg(network)
                    .arg("--memory")
                    .arg(&memory)
                    .arg("--cpus")
                    .arg(cpu.to_string())
                    .arg("--pids-limit")
                    .arg("64")
                    .arg("--read-only")
                    .arg(&image);

                for arg in &args {
                    cmd.arg(arg);
                }
                cmd.arg(&command);

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
                SandboxKind::Docker,
            ));
        }

        Ok(SandboxResult::success(stdout, SandboxKind::Docker))
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

    #[test]
    fn is_available_does_not_panic() {
        let backend = DockerSandboxBackend::new(SandboxConfig::default()).unwrap();
        let _ = backend.is_available();
    }

    #[test]
    fn supported_languages() {
        let backend = DockerSandboxBackend::new(SandboxConfig::default()).unwrap();
        let langs = backend.supported_languages();
        assert!(langs.contains(&"python".to_string()));
        assert!(langs.contains(&"javascript".to_string()));
    }

    #[test]
    fn rejects_empty_image() {
        let config = SandboxConfig {
            docker_image: Some("".into()),
            ..Default::default()
        };
        assert!(DockerSandboxBackend::new(config).is_err());
    }

    #[test]
    fn build_shell_command_python() {
        let backend = DockerSandboxBackend::new(SandboxConfig::default()).unwrap();
        let cmd = backend
            .build_shell_command("print('hi')", "python")
            .unwrap();
        assert!(cmd.contains("python3"));
    }

    #[test]
    fn build_shell_command_unsupported() {
        let backend = DockerSandboxBackend::new(SandboxConfig::default()).unwrap();
        assert!(backend.build_shell_command("code", "cobol").is_err());
    }
}
