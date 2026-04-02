use async_trait::async_trait;

use crate::descriptor::ToolDescriptor;

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

impl ToolCall {
    pub fn new(id: String, name: String, arguments: serde_json::Value) -> Self {
        Self {
            id,
            name,
            arguments,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub id: String,
    pub name: String,
    pub output: String,
    pub is_error: bool,
}

impl ToolResult {
    pub fn success(call: &ToolCall, output: impl Into<String>) -> Self {
        Self {
            id: call.id.clone(),
            name: call.name.clone(),
            output: output.into(),
            is_error: false,
        }
    }

    pub fn error(call: &ToolCall, error: impl Into<String>) -> Self {
        Self {
            id: call.id.clone(),
            name: call.name.clone(),
            output: error.into(),
            is_error: true,
        }
    }
}

#[async_trait]
pub trait ToolExecutor: Send + Sync {
    fn descriptor(&self) -> &ToolDescriptor;
    async fn execute(&self, call: ToolCall) -> ToolResult;
}
