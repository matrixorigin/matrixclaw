use async_trait::async_trait;

use super::SharedBrowserState;
use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct TypeTool {
    descriptor: ToolDescriptor,
    state: SharedBrowserState,
}

impl TypeTool {
    pub fn new(state: SharedBrowserState) -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "browser_type",
                "Type text into an element identified by CSS selector",
            )
            .with_parameters(vec![
                ToolParameter::required(
                    "selector",
                    ParameterType::String,
                    "CSS selector of the input element",
                ),
                ToolParameter::required(
                    "text",
                    ParameterType::String,
                    "Text to type into the element",
                ),
                ToolParameter::optional(
                    "clear_first",
                    ParameterType::Boolean,
                    "Clear existing content before typing (default true)",
                ),
            ]),
            state,
        }
    }
}

#[async_trait]
impl ToolExecutor for TypeTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let selector = match call.arguments.get("selector").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return ToolResult::error(&call, "missing required parameter: selector"),
        };
        let text = match call.arguments.get("text").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => return ToolResult::error(&call, "missing required parameter: text"),
        };
        let clear_first = call
            .arguments
            .get("clear_first")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        #[cfg(feature = "browser")]
        {
            let state = self.state.lock().await;
            let tab = match state.tab() {
                Ok(t) => t,
                Err(e) => return ToolResult::error(&call, e),
            };
            let clear_stmt = if clear_first { "el.value = '';" } else { "" };
            let js = format!(
                "(function() {{ var el = document.querySelector({selector:?}); if (!el) return 'Element not found: {selector}'; {clear_stmt} el.focus(); return 'focused'; }})()",
            );
            match tab.evaluate(&js, false) {
                Ok(v) if v.value.as_deref() == Some("focused") => {}
                Ok(v) => {
                    return ToolResult::error(
                        &call,
                        v.value
                            .and_then(|v| v.as_str().map(String::from))
                            .unwrap_or_else(|| "element not found".to_string()),
                    )
                }
                Err(e) => return ToolResult::error(&call, format!("focus failed: {e}")),
            }
            if let Err(e) = tab.type_str(text, false, false) {
                return ToolResult::error(&call, format!("typing failed: {e}"));
            }
            ToolResult::success(
                &call,
                format!(
                    "Typed {text_len} chars into {selector}",
                    text_len = text.len()
                ),
            )
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = (selector, text, clear_first);
            ToolResult::error(
                &call,
                "browser feature not enabled (recompile with --features browser)",
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_state() -> SharedBrowserState {
        super::super::make_shared_state(PathBuf::from("/tmp/matrixclaw-test/screenshots"))
    }

    #[test]
    fn descriptor_name() {
        let tool = TypeTool::new(make_state());
        assert_eq!(tool.descriptor().name, "browser_type");
        assert_eq!(tool.descriptor().parameters.len(), 3);
    }

    #[tokio::test]
    async fn missing_params_return_error() {
        let tool = TypeTool::new(make_state());
        let call = ToolCall::new("1".into(), "browser_type".into(), serde_json::json!({}));
        let result = tool.execute(call).await;
        assert!(result.is_error);
        assert!(result.output.contains("missing required parameter"));

        let call2 = ToolCall::new(
            "2".into(),
            "browser_type".into(),
            serde_json::json!({"selector": "#input"}),
        );
        let result2 = tool.execute(call2).await;
        assert!(result2.is_error);
        assert!(result2.output.contains("missing required parameter: text"));
    }
}
