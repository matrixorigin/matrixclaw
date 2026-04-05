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

#[derive(Clone)]
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

        let mut docker_args: Vec<String> = vec![
            "run".into(),
            "--rm".into(),
            "--network".into(),
            "none".into(),
            "--memory".into(),
            self.config.memory_limit.clone(),
            "--cpus".into(),
            self.config.cpu_limit.to_string(),
            "--pids-limit".into(),
            "64".into(),
        ];

        if self.config.read_only_root {
            docker_args.push("--read-only".into());
        }

        docker_args.extend_from_slice(&[self.config.image.clone(), "sh".into(), "-c".into(), cmd]);

        let output = if self.config.timeout_secs > 0 {
            let mut c = Command::new("timeout");
            c.arg(format!("{}s", self.config.timeout_secs));
            c.arg("docker");
            c.args(&docker_args);
            c.output()
        } else {
            let mut c = Command::new("docker");
            c.args(&docker_args);
            c.output()
        }
        .map_err(|e| format!("docker execution failed: {e}"))?;

        let exit_code = output.status.code().unwrap_or(-1);

        Ok(SandboxResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code,
            timed_out: exit_code == 124,
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
