use zstar_tools::ToolCall;

#[derive(Debug, Clone)]
pub enum RunMessageRole {
    User,
    System,
    Assistant,
    Tool,
}

#[derive(Debug, Clone)]
pub struct RunMessage {
    pub role: RunMessageRole,
    pub content: String,
    pub tool_call_id: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl RunMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: RunMessageRole::User,
            content: content.into(),
            tool_call_id: None,
            tool_calls: None,
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: RunMessageRole::System,
            content: content.into(),
            tool_call_id: None,
            tool_calls: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: RunMessageRole::Assistant,
            content: content.into(),
            tool_call_id: None,
            tool_calls: None,
        }
    }

    pub fn assistant_with_tool_calls(
        content: impl Into<String>,
        tool_calls: Vec<ToolCall>,
    ) -> Self {
        Self {
            role: RunMessageRole::Assistant,
            content: content.into(),
            tool_call_id: None,
            tool_calls: Some(tool_calls),
        }
    }

    pub fn tool_result(call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: RunMessageRole::Tool,
            content: content.into(),
            tool_call_id: Some(call_id.into()),
            tool_calls: None,
        }
    }
}
