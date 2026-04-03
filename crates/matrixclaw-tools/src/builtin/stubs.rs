use async_trait::async_trait;

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct CodeInterpreterTool {
    descriptor: ToolDescriptor,
}

impl Default for CodeInterpreterTool {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeInterpreterTool {
    pub fn new() -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "code_interpreter",
                "Execute code in a sandboxed environment (not yet implemented)",
            )
            .with_parameters(vec![
                ToolParameter::required("code", ParameterType::String, "Code to execute"),
                ToolParameter::optional("language", ParameterType::String, "Programming language"),
            ]),
        }
    }
}

#[async_trait]
impl ToolExecutor for CodeInterpreterTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        ToolResult::error(
            &call,
            "code_interpreter is not yet implemented (coming in Phase 5)",
        )
    }
}
