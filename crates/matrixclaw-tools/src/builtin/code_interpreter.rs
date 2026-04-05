use async_trait::async_trait;

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};
use crate::sandbox::{DockerSandbox, SandboxConfig};

trait EnumValues: Sized {
    fn enum_values(self, values: &[&str]) -> Self;
}

impl EnumValues for ToolParameter {
    fn enum_values(mut self, values: &[&str]) -> Self {
        self.enum_values = Some(values.iter().map(|s| s.to_string()).collect());
        self
    }
}

pub struct CodeInterpreterTool {
    descriptor: ToolDescriptor,
    sandbox: DockerSandbox,
}

impl CodeInterpreterTool {
    pub fn new() -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "code_interpreter",
                "Execute code in a sandboxed Docker container. Supports Python, JavaScript, Rust, and Bash.",
            )
            .with_parameters(vec![
                ToolParameter::required("code", ParameterType::String, "Code to execute"),
                ToolParameter::optional(
                    "language",
                    ParameterType::String,
                    "Programming language (python/javascript/rust/bash)",
                )
                .enum_values(&["python", "javascript", "rust", "bash"]),
            ]),
            sandbox: DockerSandbox::new(SandboxConfig::default()),
        }
    }
}

impl Default for CodeInterpreterTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolExecutor for CodeInterpreterTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let code = match call.arguments.get("code").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return ToolResult::error(&call, "missing required parameter: code"),
        };
        let language = call
            .arguments
            .get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("python");

        if !self.sandbox.is_available() {
            return ToolResult::error(
                &call,
                "Docker is not available. Install Docker to use code_interpreter.",
            );
        }

        let sandbox = self.sandbox.clone();
        let code_owned = code.to_string();
        let language_owned = language.to_string();
        let call_clone = call.clone();

        let result =
            tokio::task::spawn_blocking(move || sandbox.execute(&code_owned, &language_owned))
                .await
                .unwrap_or_else(|e| Err(format!("sandbox task failed: {e}")));

        match result {
            Ok(result) => {
                let mut output = result.stdout;
                if !result.stderr.is_empty() {
                    output = format!("{output}\n--- stderr ---\n{}", result.stderr);
                }
                if result.timed_out {
                    return ToolResult::error(
                        &call_clone,
                        format!("execution timed out: {output}"),
                    );
                }
                if result.exit_code != 0 {
                    return ToolResult::error(
                        &call_clone,
                        format!("exit code {}: {output}", result.exit_code),
                    );
                }
                ToolResult::success(&call_clone, output)
            }
            Err(e) => ToolResult::error(&call_clone, e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_correct() {
        let tool = CodeInterpreterTool::new();
        assert_eq!(tool.descriptor().name, "code_interpreter");
    }

    #[test]
    fn sandbox_detects_docker() {
        let sandbox = DockerSandbox::new(SandboxConfig::default());
        let _ = sandbox.is_available();
    }
}
