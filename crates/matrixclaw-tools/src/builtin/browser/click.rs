use async_trait::async_trait;

use super::SharedBrowserState;
use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct ClickTool {
    descriptor: ToolDescriptor,
    state: SharedBrowserState,
}

impl ClickTool {
    pub fn new(state: SharedBrowserState) -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "browser_click",
                "Click an element on the page by CSS selector",
            )
            .with_parameters(vec![
                ToolParameter::required(
                    "selector",
                    ParameterType::String,
                    "CSS selector of the element to click",
                ),
                ToolParameter::optional(
                    "wait_ms",
                    ParameterType::Integer,
                    "Milliseconds to wait after clicking (default 1000)",
                ),
            ]),
            state,
        }
    }
}

#[async_trait]
impl ToolExecutor for ClickTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let selector = match call.arguments.get("selector").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return ToolResult::error(&call, "missing required parameter: selector"),
        };
        let wait_ms = call
            .arguments
            .get("wait_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(1000);

        #[cfg(feature = "browser")]
        {
            let state = self.state.lock().await;
            let tab = match state.tab() {
                Ok(t) => t,
                Err(e) => return ToolResult::error(&call, e),
            };
            let js = format!(
                "(function() {{ var el = document.querySelector({selector:?}); if (!el) return 'Element not found: {selector}'; el.click(); return 'clicked'; }})()"
            );
            let result = tab.evaluate(&js, false);
            match result {
                Ok(v) if v.value.as_deref() == Some("clicked") => {
                    std::thread::sleep(std::time::Duration::from_millis(wait_ms));
                    ToolResult::success(&call, format!("Clicked element: {selector}"))
                }
                Ok(v) => ToolResult::error(
                    &call,
                    v.value
                        .and_then(|v| v.as_str().map(String::from))
                        .unwrap_or_else(|| "unknown click error".to_string()),
                ),
                Err(e) => ToolResult::error(&call, format!("click failed: {e}")),
            }
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = (selector, wait_ms);
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
        let tool = ClickTool::new(make_state());
        assert_eq!(tool.descriptor().name, "browser_click");
        assert_eq!(tool.descriptor().parameters.len(), 2);
    }

    #[tokio::test]
    async fn missing_selector_returns_error() {
        let tool = ClickTool::new(make_state());
        let call = ToolCall::new("1".into(), "browser_click".into(), serde_json::json!({}));
        let result = tool.execute(call).await;
        assert!(result.is_error);
        assert!(result
            .output
            .contains("missing required parameter: selector"));
    }
}
