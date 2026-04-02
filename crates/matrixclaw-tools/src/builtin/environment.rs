use async_trait::async_trait;

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct EnvironmentTool {
    descriptor: ToolDescriptor,
}

impl Default for EnvironmentTool {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentTool {
    pub fn new() -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "environment",
                "Get environment variables and system information",
            )
            .with_parameters(vec![
                ToolParameter::required("action", ParameterType::String, "Action to perform")
                    .enum_values(&["get_var", "list_vars", "system_info"]),
                ToolParameter::optional(
                    "name",
                    ParameterType::String,
                    "Name of environment variable (for get_var)",
                ),
            ]),
        }
    }
}

impl ToolParameter {
    fn enum_values(mut self, values: &[&str]) -> Self {
        self.enum_values = Some(values.iter().map(|s| s.to_string()).collect());
        self
    }
}

#[async_trait]
impl ToolExecutor for EnvironmentTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let action = match call.arguments.get("action").and_then(|v| v.as_str()) {
            Some(a) => a,
            None => return ToolResult::error(&call, "missing required parameter: action"),
        };

        match action {
            "get_var" => {
                let name = match call.arguments.get("name").and_then(|v| v.as_str()) {
                    Some(n) => n,
                    None => return ToolResult::error(&call, "missing required parameter: name"),
                };
                match std::env::var(name) {
                    Ok(val) => ToolResult::success(&call, val),
                    Err(_) => {
                        ToolResult::error(&call, format!("environment variable not found: {name}"))
                    }
                }
            }
            "list_vars" => {
                let vars: Vec<String> = std::env::vars().map(|(k, _)| k).collect();
                ToolResult::success(&call, vars.join("\n"))
            }
            "system_info" => {
                let os = std::env::consts::OS.to_string();
                let arch = std::env::consts::ARCH.to_string();
                let hostname = hostname::get()
                    .map(|h| h.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "unknown".to_string());
                let cwd = std::env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "unknown".to_string());
                ToolResult::success(
                    &call,
                    format!("os: {os}\narch: {arch}\nhostname: {hostname}\ncwd: {cwd}"),
                )
            }
            _ => ToolResult::error(&call, format!("unknown action: {action}")),
        }
    }
}

mod hostname {
    use std::ffi::OsString;

    pub fn get() -> Result<OsString, std::io::Error> {
        let output = std::process::Command::new("hostname").output()?;
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(OsString::from(name))
    }
}
