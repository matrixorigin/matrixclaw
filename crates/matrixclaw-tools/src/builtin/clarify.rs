use std::io::Write;

use async_trait::async_trait;

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct ClarifyTool {
    descriptor: ToolDescriptor,
}

impl Default for ClarifyTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ClarifyTool {
    pub fn new() -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "clarify",
                "Ask the user a question to clarify intent. Shows options if provided.",
            )
            .with_parameters(vec![
                ToolParameter::required(
                    "question",
                    ParameterType::String,
                    "Question to ask the user",
                ),
                ToolParameter::optional(
                    "options",
                    ParameterType::String,
                    "Comma-separated list of options (up to 4)",
                ),
            ]),
        }
    }
}

#[async_trait]
impl ToolExecutor for ClarifyTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let question = match call.arguments.get("question").and_then(|v| v.as_str()) {
            Some(q) => q,
            None => return ToolResult::error(&call, "missing required parameter: question"),
        };

        let options: Vec<String> = call
            .arguments
            .get("options")
            .and_then(|v| v.as_str())
            .map(|s| {
                s.split(',')
                    .map(|o| o.trim().to_string())
                    .filter(|o| !o.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        if !options.is_empty() {
            eprintln!("\n❓ {question}");
            for (i, opt) in options.iter().enumerate() {
                eprintln!("  {}. {opt}", i + 1);
            }
        } else {
            eprintln!("\n❓ {question}\n  (type your answer)");
        }

        eprint!("> ");
        let _ = std::io::stderr().flush();

        let mut buf = String::new();
        match std::io::stdin().read_line(&mut buf) {
            Ok(_) => ToolResult::success(&call, buf.trim().to_string()),
            Err(_) => ToolResult::error(&call, "clarify requires interactive terminal"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_has_correct_name() {
        let tool = ClarifyTool::new();
        assert_eq!(tool.descriptor().name, "clarify");
        assert_eq!(tool.descriptor().parameters.len(), 2);
    }
}
