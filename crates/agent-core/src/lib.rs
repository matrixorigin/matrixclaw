pub mod approval;
pub mod event;
pub mod r#loop;
pub mod message;
pub mod policy;
pub mod provider;
pub mod tool;

pub use matrixclaw_tools::{ToolCall, ToolDescriptor, ToolResult};
pub use message::{RunMessage, RunMessageRole};

#[derive(Debug, Clone)]
pub struct RunRequest {
    pub prompt: String,
    pub context_messages: Vec<RunMessage>,
    pub tools: Vec<ToolDescriptor>,
    pub tool_choice: ToolChoice,
    pub max_iterations: u32,
}

impl RunRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            context_messages: Vec::new(),
            tools: Vec::new(),
            tool_choice: ToolChoice::Auto,
            max_iterations: 90,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ToolChoice {
    Auto,
    None,
}

#[derive(Debug, Clone)]
pub struct RunResult {
    pub streamed_message: String,
    pub final_message: String,
    pub tool_calls_made: u32,
    pub iterations: u32,
}

pub use r#loop::{run_prompt, run_prompt_with_policy};
