pub mod event;
pub mod r#loop;
pub mod message;
pub mod policy;
pub mod provider;
pub mod tool;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunMessageRole {
    User,
    System,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunMessage {
    pub role: RunMessageRole,
    pub content: String,
}

impl RunMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: RunMessageRole::User,
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: RunMessageRole::System,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: RunMessageRole::Assistant,
            content: content.into(),
        }
    }

    pub fn tool(content: impl Into<String>) -> Self {
        Self {
            role: RunMessageRole::Tool,
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRequest {
    pub prompt: String,
    pub context_messages: Vec<RunMessage>,
}

impl RunRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            context_messages: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunResult {
    pub streamed_message: String,
    pub final_message: String,
}

pub use r#loop::run_prompt;
