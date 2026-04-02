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

pub struct DelegateTool {
    descriptor: ToolDescriptor,
}

impl Default for DelegateTool {
    fn default() -> Self {
        Self::new()
    }
}

impl DelegateTool {
    pub fn new() -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "delegate",
                "Delegate a task to a sub-agent (not yet implemented)",
            )
            .with_parameters(vec![
                ToolParameter::required("task", ParameterType::String, "Task description"),
                ToolParameter::optional(
                    "agent_profile",
                    ParameterType::String,
                    "Agent profile to use",
                ),
            ]),
        }
    }
}

#[async_trait]
impl ToolExecutor for DelegateTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        ToolResult::error(&call, "delegate is not yet implemented (coming in Phase 3)")
    }
}

pub struct SkillsTool {
    descriptor: ToolDescriptor,
}

impl Default for SkillsTool {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillsTool {
    pub fn new() -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "skills",
                "List, read, and create skills (not yet implemented)",
            )
            .with_parameters(vec![
                ToolParameter::required("action", ParameterType::String, "Action to perform"),
                ToolParameter::optional("name", ParameterType::String, "Skill name"),
            ]),
        }
    }
}

#[async_trait]
impl ToolExecutor for SkillsTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        ToolResult::error(&call, "skills is not yet implemented (coming in Phase 4)")
    }
}
