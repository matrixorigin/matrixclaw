use crate::message::{ToolCallMessage, ToolResultMessage};
use matrixclaw_manifests::config::ExecutionBackendSelection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolExecutionMode {
    Local,
    Sandboxed,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolExecutionBackendKind {
    LocalCommand,
    Sandbox,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolExecutionBackendSelection {
    pub mode: ToolExecutionMode,
    pub kind: ToolExecutionBackendKind,
    pub label: String,
    pub requires_docker: bool,
}

impl ToolExecutionBackendSelection {
    pub fn local_default() -> Self {
        Self {
            mode: ToolExecutionMode::Local,
            kind: ToolExecutionBackendKind::LocalCommand,
            label: "local-command".to_string(),
            requires_docker: false,
        }
    }

    pub fn sandbox() -> Self {
        Self::sandbox_with_label("sandbox", false)
    }

    pub fn sandbox_with_label(label: impl Into<String>, requires_docker: bool) -> Self {
        Self {
            mode: ToolExecutionMode::Sandboxed,
            kind: ToolExecutionBackendKind::Sandbox,
            label: label.into(),
            requires_docker,
        }
    }

    pub fn disabled() -> Self {
        Self::disabled_with_label("disabled", false)
    }

    pub fn disabled_with_label(label: impl Into<String>, requires_docker: bool) -> Self {
        Self {
            mode: ToolExecutionMode::Disabled,
            kind: ToolExecutionBackendKind::Disabled,
            label: label.into(),
            requires_docker,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredExecutionResult {
    pub backend: ExecutionBackendSelection,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl StructuredExecutionResult {
    pub fn new(
        backend: ExecutionBackendSelection,
        exit_code: i32,
        stdout: impl Into<String>,
        stderr: impl Into<String>,
    ) -> Self {
        Self {
            backend,
            exit_code,
            stdout: stdout.into(),
            stderr: stderr.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolExecutionRequest {
    pub call: ToolCallMessage,
}

impl ToolExecutionRequest {
    pub fn new(call: ToolCallMessage) -> Self {
        Self { call }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolExecutionResponse {
    pub result: ToolResultMessage,
}

impl ToolExecutionResponse {
    pub fn new(result: ToolResultMessage) -> Self {
        Self { result }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockedToolResult {
    pub result: ToolResultMessage,
    pub reason: String,
}

impl BlockedToolResult {
    pub fn new(call: ToolCallMessage, reason: impl Into<String>) -> Self {
        let reason = reason.into();
        let result = ToolResultMessage::new(call.tool_name, format!("blocked: {}", reason));
        Self { result, reason }
    }
}

pub trait ToolExecutor {
    fn execute(&mut self, request: &ToolExecutionRequest) -> ToolExecutionResponse;
}

pub fn parse_tool_calls(output: &str) -> Vec<ToolCallMessage> {
    output
        .lines()
        .filter_map(|line| parse_tool_call_line(line.trim()))
        .collect()
}

pub fn synthesize_tool_continuation(results: &[ToolResultMessage]) -> String {
    results
        .iter()
        .map(ToolResultMessage::as_assistant_fragment)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn build_tool_result_prompt(results: &[ToolResultMessage]) -> String {
    synthesize_tool_continuation(results)
}

fn parse_tool_call_line(line: &str) -> Option<ToolCallMessage> {
    let call = line.strip_prefix("call:")?;
    let open_paren = call.find('(')?;
    let close_paren = call.rfind(')')?;
    if close_paren <= open_paren {
        return None;
    }

    let tool_name = call[..open_paren].trim();
    let arguments = call[open_paren + 1..close_paren].trim();
    if tool_name.is_empty() {
        return None;
    }

    Some(ToolCallMessage::new(tool_name, arguments))
}
