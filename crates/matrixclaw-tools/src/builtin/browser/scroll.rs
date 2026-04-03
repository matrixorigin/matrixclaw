use async_trait::async_trait;

use super::SharedBrowserState;
use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct ScrollTool {
    descriptor: ToolDescriptor,
    state: SharedBrowserState,
}

impl ScrollTool {
    pub fn new(state: SharedBrowserState) -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "browser_scroll",
                "Scroll the page by a relative amount in pixels",
            )
            .with_parameters(vec![
                ToolParameter::optional(
                    "direction",
                    ParameterType::String,
                    "Scroll direction: up or down (default down)",
                ),
                ToolParameter::optional(
                    "amount",
                    ParameterType::Integer,
                    "Pixels to scroll (default 500)",
                ),
            ]),
            state,
        }
    }
}

#[async_trait]
impl ToolExecutor for ScrollTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let direction = call
            .arguments
            .get("direction")
            .and_then(|v| v.as_str())
            .unwrap_or("down");
        let amount = call
            .arguments
            .get("amount")
            .and_then(|v| v.as_i64())
            .unwrap_or(500);

        let signed_amount = match direction {
            "down" => amount,
            "up" => -amount,
            _ => {
                return ToolResult::error(
                    &call,
                    format!("invalid direction: {direction} (use up or down)"),
                )
            }
        };

        #[cfg(feature = "browser")]
        {
            let state = self.state.lock().await;
            let tab = match state.tab() {
                Ok(t) => t,
                Err(e) => return ToolResult::error(&call, e),
            };
            let js = format!("window.scrollBy(0, {signed_amount})");
            if let Err(e) = tab.evaluate(&js, false) {
                return ToolResult::error(&call, format!("scroll failed: {e}"));
            }
            ToolResult::success(&call, format!("Scrolled {direction} by {amount}px"))
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = signed_amount;
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
        let tool = ScrollTool::new(make_state());
        assert_eq!(tool.descriptor().name, "browser_scroll");
        assert_eq!(tool.descriptor().parameters.len(), 2);
        assert!(!tool.descriptor().parameters[0].required);
        assert!(!tool.descriptor().parameters[1].required);
    }

    #[tokio::test]
    async fn invalid_direction_returns_error() {
        let tool = ScrollTool::new(make_state());
        let call = ToolCall::new(
            "1".into(),
            "browser_scroll".into(),
            serde_json::json!({"direction": "sideways"}),
        );
        let result = tool.execute(call).await;
        assert!(result.is_error);
        assert!(result.output.contains("invalid direction"));
    }
}
