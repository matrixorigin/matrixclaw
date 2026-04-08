use async_trait::async_trait;
use sandwrench::runtime::CodeRequest;
use sandwrench::{SandboxConfig, SandboxProvider};

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

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
    provider: SandboxProvider,
}

impl CodeInterpreterTool {
    pub fn new() -> Self {
        Self::with_config(SandboxConfig::default()).expect("failed to create sandbox provider")
    }

    pub fn with_config(config: SandboxConfig) -> Result<Self, sandwrench::SandboxError> {
        let provider = SandboxProvider::from_config(&config)?;
        Ok(Self {
            descriptor: ToolDescriptor::new(
                "code_interpreter",
                "Execute code in a sandboxed environment. Supports Python, JavaScript, Rust, and Bash.",
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
            provider,
        })
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
            Some(c) => c.to_string(),
            None => return ToolResult::error(&call, "missing required parameter: code"),
        };
        let language = call
            .arguments
            .get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("python")
            .to_string();

        if !self.provider.is_available() {
            return ToolResult::error(
                &call,
                "Sandbox backend is not available. Install Docker or configure a cloud sandbox.",
            );
        }

        let request = CodeRequest {
            code,
            language,
            timeout_secs: None,
        };

        match self.provider.execute_code(request).await {
            Ok(result) => {
                let mut output = result.stdout;
                if !result.stderr.is_empty() {
                    output = format!("{output}\n--- stderr ---\n{}", result.stderr);
                }
                if result.timed_out {
                    return ToolResult::error(&call, format!("execution timed out: {output}"));
                }
                if result.exit_code != 0 {
                    return ToolResult::error(
                        &call,
                        format!("exit code {}: {output}", result.exit_code),
                    );
                }
                ToolResult::success(&call, output)
            }
            Err(e) => ToolResult::error(&call, format!("sandbox error: {e}")),
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
    fn local_backend_available() {
        let config = SandboxConfig {
            kind: sandwrench::SandboxKind::Local,
            ..SandboxConfig::default()
        };
        let tool = CodeInterpreterTool::with_config(config).unwrap();
        assert!(tool.provider.is_available());
    }
}
