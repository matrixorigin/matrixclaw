use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct MemoryTool {
    descriptor: ToolDescriptor,
    store: Arc<Mutex<HashMap<String, String>>>,
}

impl Default for MemoryTool {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryTool {
    pub fn new() -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "memory",
                "Key-value memory store for persisting information across tool calls",
            )
            .with_parameters(vec![
                ToolParameter::required("action", ParameterType::String, "Action to perform")
                    .enum_values(&["store", "retrieve", "list", "delete"]),
                ToolParameter::optional("key", ParameterType::String, "Key for the memory entry"),
                ToolParameter::optional("value", ParameterType::String, "Value to store"),
            ]),
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
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

#[async_trait]
impl ToolExecutor for MemoryTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let action = match call.arguments.get("action").and_then(|v| v.as_str()) {
            Some(a) => a,
            None => return ToolResult::error(&call, "missing required parameter: action"),
        };

        match action {
            "store" => {
                let key = match call.arguments.get("key").and_then(|v| v.as_str()) {
                    Some(k) => k.to_string(),
                    None => return ToolResult::error(&call, "missing required parameter: key"),
                };
                let value = match call.arguments.get("value").and_then(|v| v.as_str()) {
                    Some(v) => v.to_string(),
                    None => return ToolResult::error(&call, "missing required parameter: value"),
                };
                let mut store = self.store.lock().unwrap();
                store.insert(key.clone(), value);
                ToolResult::success(&call, format!("stored {key}"))
            }
            "retrieve" => {
                let key = match call.arguments.get("key").and_then(|v| v.as_str()) {
                    Some(k) => k,
                    None => return ToolResult::error(&call, "missing required parameter: key"),
                };
                let store = self.store.lock().unwrap();
                match store.get(key) {
                    Some(val) => ToolResult::success(&call, val.clone()),
                    None => ToolResult::error(&call, format!("key not found: {key}")),
                }
            }
            "list" => {
                let store = self.store.lock().unwrap();
                let keys: Vec<String> = store.keys().cloned().collect();
                if keys.is_empty() {
                    ToolResult::success(&call, "(empty)")
                } else {
                    ToolResult::success(&call, keys.join("\n"))
                }
            }
            "delete" => {
                let key = match call.arguments.get("key").and_then(|v| v.as_str()) {
                    Some(k) => k,
                    None => return ToolResult::error(&call, "missing required parameter: key"),
                };
                let mut store = self.store.lock().unwrap();
                match store.remove(key) {
                    Some(_) => ToolResult::success(&call, format!("deleted {key}")),
                    None => ToolResult::error(&call, format!("key not found: {key}")),
                }
            }
            _ => ToolResult::error(&call, format!("unknown action: {action}")),
        }
    }
}
