use async_trait::async_trait;

use super::SharedBrowserState;
use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct NavigateTool {
    descriptor: ToolDescriptor,
    state: SharedBrowserState,
}

impl NavigateTool {
    pub fn new(state: SharedBrowserState) -> Self {
        Self {
            descriptor: ToolDescriptor::new("browser_navigate", "Navigate the browser to a URL")
                .with_parameters(vec![
                    ToolParameter::required("url", ParameterType::String, "URL to navigate to"),
                    ToolParameter::optional(
                        "wait_ms",
                        ParameterType::Integer,
                        "Milliseconds to wait for page load (default 2000)",
                    ),
                ]),
            state,
        }
    }
}

#[async_trait]
impl ToolExecutor for NavigateTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let url = match call.arguments.get("url").and_then(|v| v.as_str()) {
            Some(u) => u,
            None => return ToolResult::error(&call, "missing required parameter: url"),
        };
        let wait_ms = call
            .arguments
            .get("wait_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(2000);

        #[cfg(feature = "browser")]
        {
            let mut state = self.state.lock().await;
            if let Err(e) = state.ensure_browser().await {
                return ToolResult::error(&call, e);
            }
            let tab = match state.tab() {
                Ok(t) => t,
                Err(e) => return ToolResult::error(&call, e),
            };
            if let Err(e) = tab.navigate_to(url) {
                return ToolResult::error(&call, format!("navigation failed: {e}"));
            }
            std::thread::sleep(std::time::Duration::from_millis(wait_ms));
            let title = tab.get_title().unwrap_or_else(|_| "unknown".to_string());
            ToolResult::success(&call, format!("Navigated to {url}\nTitle: {title}"))
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = (url, wait_ms);
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
        let tool = NavigateTool::new(make_state());
        assert_eq!(tool.descriptor().name, "browser_navigate");
        assert_eq!(tool.descriptor().parameters.len(), 2);
    }

    #[tokio::test]
    async fn missing_url_returns_error() {
        let tool = NavigateTool::new(make_state());
        let call = ToolCall::new("1".into(), "browser_navigate".into(), serde_json::json!({}));
        let result = tool.execute(call).await;
        assert!(result.is_error);
        assert!(result.output.contains("missing required parameter: url"));
    }
}
