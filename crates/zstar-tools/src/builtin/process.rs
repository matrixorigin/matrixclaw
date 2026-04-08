use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use async_trait::async_trait;

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

#[derive(Debug, Clone)]
struct ProcessEntry {
    pid: u32,
    command: String,
    started_at: String,
    status: String,
}

pub struct ProcessTool {
    descriptor: ToolDescriptor,
    processes: Arc<Mutex<HashMap<u32, ProcessEntry>>>,
}

trait EnumValues {
    fn enum_values(self, values: &[&str]) -> Self;
}

impl EnumValues for ToolParameter {
    fn enum_values(mut self, values: &[&str]) -> Self {
        self.enum_values = Some(values.iter().map(|s| s.to_string()).collect());
        self
    }
}

impl Default for ProcessTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessTool {
    pub fn new() -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "process",
                "Manage background processes: list running processes, register new ones, kill them.",
            )
            .with_parameters(vec![
                ToolParameter::required("action", ParameterType::String, "Action to perform")
                    .enum_values(&["list", "register", "kill"]),
                ToolParameter::optional("pid", ParameterType::String, "Process ID"),
                ToolParameter::optional("command", ParameterType::String, "Command that was run"),
                ToolParameter::optional("status", ParameterType::String, "Process status"),
            ]),
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ToolExecutor for ProcessTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let action = match call.arguments.get("action").and_then(|v| v.as_str()) {
            Some(a) => a,
            None => return ToolResult::error(&call, "missing required parameter: action"),
        };

        match action {
            "list" => {
                let processes = match self.processes.lock() {
                    Ok(p) => p,
                    Err(e) => return ToolResult::error(&call, format!("process table lock: {e}")),
                };
                if processes.is_empty() {
                    return ToolResult::success(&call, "(no tracked processes)");
                }
                let lines: Vec<String> = processes
                    .values()
                    .map(|p| {
                        format!(
                            "{}: {} ({}, started {})",
                            p.pid, p.command, p.status, p.started_at
                        )
                    })
                    .collect();
                ToolResult::success(&call, lines.join("\n"))
            }
            "register" => {
                let pid = match call.arguments.get("pid").and_then(|v| v.as_str()) {
                    Some(p) => match p.parse::<u32>() {
                        Ok(n) => n,
                        Err(_) => return ToolResult::error(&call, "invalid pid"),
                    },
                    None => return ToolResult::error(&call, "missing required parameter: pid"),
                };
                let command = match call.arguments.get("command").and_then(|v| v.as_str()) {
                    Some(c) => c.to_string(),
                    None => return ToolResult::error(&call, "missing required parameter: command"),
                };
                let status = call
                    .arguments
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("running")
                    .to_string();
                let started_at = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map(|d| d.as_secs().to_string())
                    .unwrap_or_else(|_| "unknown".to_string());
                let entry = ProcessEntry {
                    pid,
                    command,
                    started_at,
                    status,
                };
                {
                    let mut processes = match self.processes.lock() {
                        Ok(p) => p,
                        Err(e) => {
                            return ToolResult::error(&call, format!("process table lock: {e}"))
                        }
                    };
                    processes.insert(pid, entry);
                }
                ToolResult::success(&call, format!("tracking process {pid}"))
            }
            "kill" => {
                let pid = match call.arguments.get("pid").and_then(|v| v.as_str()) {
                    Some(p) => match p.parse::<u32>() {
                        Ok(n) => n,
                        Err(_) => return ToolResult::error(&call, "invalid pid"),
                    },
                    None => return ToolResult::error(&call, "missing required parameter: pid"),
                };
                let existed;
                {
                    let mut processes = match self.processes.lock() {
                        Ok(p) => p,
                        Err(e) => {
                            return ToolResult::error(&call, format!("process table lock: {e}"))
                        }
                    };
                    existed = processes.remove(&pid).is_some();
                }
                if !existed {
                    return ToolResult::error(&call, format!("process {pid} not found"));
                }
                let kill_result = Command::new("kill")
                    .arg("-TERM")
                    .arg(pid.to_string())
                    .output();
                match kill_result {
                    Ok(output) if output.status.success() => {
                        ToolResult::success(&call, format!("killed process {pid}"))
                    }
                    Ok(_) => ToolResult::success(
                        &call,
                        format!("killed process {pid} (signal may have failed)"),
                    ),
                    Err(e) => ToolResult::success(
                        &call,
                        format!("killed process {pid} (kill command failed: {e})"),
                    ),
                }
            }
            _ => ToolResult::error(&call, format!("unknown action: {action}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn call(tool: &ProcessTool, args: &str) -> ToolResult {
        let call = ToolCall::new(
            "1".into(),
            "process".into(),
            serde_json::json!(serde_json::from_str::<serde_json::Value>(args).unwrap()),
        );
        tool.execute(call).await
    }

    #[tokio::test]
    async fn list_empty() {
        let tool = ProcessTool::new();
        let r = call(&tool, r#"{"action":"list"}"#).await;
        assert!(!r.is_error);
        assert_eq!(r.output, "(no tracked processes)");
    }

    #[tokio::test]
    async fn register_and_list() {
        let tool = ProcessTool::new();
        let r = call(
            &tool,
            r#"{"action":"register","pid":"12345","command":"sleep 100"}"#,
        )
        .await;
        assert!(!r.is_error);
        assert!(r.output.contains("tracking process 12345"));
        let r = call(&tool, r#"{"action":"list"}"#).await;
        assert!(!r.is_error);
        assert!(r.output.contains("12345"));
        assert!(r.output.contains("sleep 100"));
        assert!(r.output.contains("running"));
    }

    #[tokio::test]
    async fn register_and_kill() {
        let tool = ProcessTool::new();
        call(
            &tool,
            r#"{"action":"register","pid":"99999999","command":"fake"}"#,
        )
        .await;
        let r = call(&tool, r#"{"action":"kill","pid":"99999999"}"#).await;
        assert!(!r.is_error);
        let r = call(&tool, r#"{"action":"list"}"#).await;
        assert_eq!(r.output, "(no tracked processes)");
    }

    #[tokio::test]
    async fn kill_nonexistent() {
        let tool = ProcessTool::new();
        let r = call(&tool, r#"{"action":"kill","pid":"1"}"#).await;
        assert!(r.is_error);
        assert!(r.output.contains("not found"));
    }
}
