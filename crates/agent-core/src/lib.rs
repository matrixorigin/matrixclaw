pub mod event;
pub mod r#loop;
pub mod message;
pub mod policy;
pub mod provider;
pub mod tool;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRequest {
    pub prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunResult {
    pub streamed_message: String,
    pub final_message: String,
}

pub use r#loop::run_prompt;
