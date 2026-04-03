use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub image: String,
    pub memory_limit: String,
    pub cpu_limit: f64,
    pub timeout_secs: u64,
    pub read_only_root: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            image: "ubuntu:22.04".to_string(),
            memory_limit: "512m".to_string(),
            cpu_limit: 1.0,
            timeout_secs: 30,
            read_only_root: true,
        }
    }
}

pub struct DockerSandbox {
    config: SandboxConfig,
}

impl DockerSandbox {
    pub fn new(config: SandboxConfig) -> Self {
        Self { config }
    }

    pub fn is_available(&self) -> bool {
        Command::new("docker")
            .arg("info")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn execute(&self, code: &str, language: &str) -> Result<SandboxResult, String> {
        let cmd = match language {
            "python" | "py" => {
                format!("echo '{}' | python3", code.replace('\'', "'\\''"))
            }
            "rust" | "rs" => {
                format!(
                    "echo '{}' | rustc - -o /tmp/script && /tmp/script",
                    code.replace('\'', "'\\''")
                )
            }
            "javascript" | "js" => {
                format!("echo '{}' | node", code.replace('\'', "'\\''"))
            }
            "bash" | "sh" => code.replace('\'', "'\\''"),
            _ => return Err(format!("unsupported language: {language}")),
        };

        let mut docker_cmd = Command::new("docker");
        docker_cmd
            .arg("run")
            .arg("--rm")
            .arg("--network")
            .arg("none")
            .arg("--memory")
            .arg(&self.config.memory_limit)
            .arg("--cpus")
            .arg(self.config.cpu_limit.to_string())
            .arg("--pids-limit")
            .arg("64");

        if self.config.read_only_root {
            docker_cmd.arg("--read-only");
        }

        docker_cmd
            .arg(&self.config.image)
            .arg("sh")
            .arg("-c")
            .arg(&cmd);

        let output = docker_cmd
            .output()
            .map_err(|e| format!("docker execution failed: {e}"))?;

        Ok(SandboxResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            timed_out: false,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SandboxResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
}
