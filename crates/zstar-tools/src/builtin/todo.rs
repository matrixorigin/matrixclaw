use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct TodoTool {
    descriptor: ToolDescriptor,
    items: Arc<Mutex<Vec<TodoItem>>>,
}

#[derive(Debug, Clone)]
struct TodoItem {
    id: usize,
    text: String,
    status: String,
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

impl TodoTool {
    pub fn new() -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "todo",
                "Session-scoped task list for tracking multi-step work. Use for 3+ step tasks.",
            )
            .with_parameters(vec![
                ToolParameter::required("action", ParameterType::String, "Action to perform")
                    .enum_values(&["list", "add", "update", "remove"]),
                ToolParameter::optional("id", ParameterType::String, "Task ID number"),
                ToolParameter::optional("text", ParameterType::String, "Task description"),
                ToolParameter::optional("status", ParameterType::String, "Task status")
                    .enum_values(&["pending", "in_progress", "done"]),
            ]),
            items: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Default for TodoTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolExecutor for TodoTool {
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
                let items = self.items.lock().unwrap();
                if items.is_empty() {
                    ToolResult::success(&call, "(no tasks)")
                } else {
                    let output: Vec<String> = items
                        .iter()
                        .map(|item| format!("{}. [{}] {}", item.id, item.status, item.text))
                        .collect();
                    ToolResult::success(&call, output.join("\n"))
                }
            }
            "add" => {
                let text = match call.arguments.get("text").and_then(|v| v.as_str()) {
                    Some(t) => t.to_string(),
                    None => return ToolResult::error(&call, "missing required parameter: text"),
                };
                let mut items = self.items.lock().unwrap();
                let id = items.len() + 1;
                let item = TodoItem {
                    id,
                    text,
                    status: "pending".to_string(),
                };
                let output = format!("{}. [{}] {}", item.id, item.status, item.text);
                items.push(item);
                ToolResult::success(&call, output)
            }
            "update" => {
                let id: usize = match call.arguments.get("id").and_then(|v| v.as_str()) {
                    Some(s) => match s.parse() {
                        Ok(n) => n,
                        Err(_) => return ToolResult::error(&call, "id must be a number"),
                    },
                    None => return ToolResult::error(&call, "missing required parameter: id"),
                };
                let mut items = self.items.lock().unwrap();
                let item = match items.iter_mut().find(|i| i.id == id) {
                    Some(i) => i,
                    None => return ToolResult::error(&call, format!("task not found: {id}")),
                };
                if let Some(text) = call.arguments.get("text").and_then(|v| v.as_str()) {
                    item.text = text.to_string();
                }
                if let Some(status) = call.arguments.get("status").and_then(|v| v.as_str()) {
                    item.status = status.to_string();
                }
                ToolResult::success(
                    &call,
                    format!("{}. [{}] {}", item.id, item.status, item.text),
                )
            }
            "remove" => {
                let id: usize = match call.arguments.get("id").and_then(|v| v.as_str()) {
                    Some(s) => match s.parse() {
                        Ok(n) => n,
                        Err(_) => return ToolResult::error(&call, "id must be a number"),
                    },
                    None => return ToolResult::error(&call, "missing required parameter: id"),
                };
                let mut items = self.items.lock().unwrap();
                let len_before = items.len();
                items.retain(|i| i.id != id);
                if items.len() == len_before {
                    return ToolResult::error(&call, format!("task not found: {id}"));
                }
                ToolResult::success(&call, format!("removed task {id}"))
            }
            _ => ToolResult::error(&call, format!("unknown action: {action}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn call(tool: &TodoTool, args: &str) -> ToolResult {
        let call = ToolCall::new(
            "1".into(),
            "todo".into(),
            serde_json::json!(serde_json::from_str::<serde_json::Value>(args).unwrap()),
        );
        tool.execute(call).await
    }

    #[tokio::test]
    async fn add_and_list() {
        let tool = TodoTool::new();
        let r = call(&tool, r#"{"action":"add","text":"Fix bug"}"#).await;
        assert!(!r.is_error);
        let r = call(&tool, r#"{"action":"list"}"#).await;
        assert!(!r.is_error);
        assert!(r.output.contains("Fix bug"));
    }

    #[tokio::test]
    async fn update_status() {
        let tool = TodoTool::new();
        call(&tool, r#"{"action":"add","text":"Write tests"}"#).await;
        let r = call(&tool, r#"{"action":"update","id":"1","status":"done"}"#).await;
        assert!(!r.is_error);
        let r = call(&tool, r#"{"action":"list"}"#).await;
        assert!(r.output.contains("[done]"));
    }

    #[tokio::test]
    async fn remove_item() {
        let tool = TodoTool::new();
        call(&tool, r#"{"action":"add","text":"Remove me"}"#).await;
        let r = call(&tool, r#"{"action":"remove","id":"1"}"#).await;
        assert!(!r.is_error);
        let r = call(&tool, r#"{"action":"list"}"#).await;
        assert_eq!(r.output, "(no tasks)");
    }

    #[tokio::test]
    async fn list_empty() {
        let tool = TodoTool::new();
        let r = call(&tool, r#"{"action":"list"}"#).await;
        assert!(!r.is_error);
        assert_eq!(r.output, "(no tasks)");
    }

    #[tokio::test]
    async fn auto_increment_ids() {
        let tool = TodoTool::new();
        call(&tool, r#"{"action":"add","text":"First"}"#).await;
        call(&tool, r#"{"action":"add","text":"Second"}"#).await;
        call(&tool, r#"{"action":"add","text":"Third"}"#).await;
        let r = call(&tool, r#"{"action":"list"}"#).await;
        assert!(r.output.contains("1. [pending] First"));
        assert!(r.output.contains("2. [pending] Second"));
        assert!(r.output.contains("3. [pending] Third"));
    }
}
