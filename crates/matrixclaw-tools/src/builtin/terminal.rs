use async_trait::async_trait;

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct TerminalTool {
    descriptor: ToolDescriptor,
    workspace_root: String,
}

impl TerminalTool {
    pub fn new(workspace_root: &str) -> Self {
        Self {
            descriptor: ToolDescriptor::new("terminal", "Execute shell commands").with_parameters(vec![
                ToolParameter::required("command", ParameterType::String, "The shell command to execute"),
                ToolParameter::optional("cwd", ParameterType::String, "Working directory for the command"),
                ToolParameter::optional("timeout", ParameterType::Integer, "Timeout in seconds (default 30)"),
            ]),
            workspace_root: workspace_root.to_string(),
        }
    }
}

#[async_trait]
impl ToolExecutor for TerminalTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let command = match call.arguments.get("command").and_then(|v| v.as_str()) {
            Some(c) => c.to_string(),
            None => return ToolResult::error(&call, "missing required parameter: command"),
        };

        let cwd = call
            .arguments
            .get("cwd")
            .and_then(|v| v.as_str())
            .unwrap_or(&self.workspace_root)
            .to_string();

        let timeout_secs = call
            .arguments
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            tokio::process::Command::new("sh")
                .arg("-c")
                .arg(&command)
                .current_dir(&cwd)
                .output(),
        )
        .await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let mut combined = String::new();
                if !stdout.is_empty() {
                    combined.push_str(&stdout);
                }
                if !stderr.is_empty() {
                    if !combined.is_empty() {
                        combined.push('\n');
                    }
                    combined.push_str(&stderr);
                }
                if output.status.success() {
                    ToolResult::success(&call, combined)
                } else {
                    ToolResult::error(
                        &call,
                        format!("exit code {}: {}", output.status.code().unwrap_or(-1), combined),
                    )
                }
            }
            Ok(Err(e)) => ToolResult::error(&call, format!("failed to execute command: {}", e)),
            Err(_) => ToolResult::error(&call, format!("command timed out after {}s", timeout_secs)),
        }
    }
}
