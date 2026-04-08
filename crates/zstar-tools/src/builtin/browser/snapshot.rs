use async_trait::async_trait;

use super::SharedBrowserState;
use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct SnapshotTool {
    descriptor: ToolDescriptor,
    state: SharedBrowserState,
}

impl SnapshotTool {
    pub fn new(state: SharedBrowserState) -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "browser_snapshot",
                "Capture the accessibility tree or text content of the current page",
            )
            .with_parameters(vec![ToolParameter::optional(
                "selector",
                ParameterType::String,
                "CSS selector to scope the snapshot (default: full page)",
            )]),
            state,
        }
    }
}

#[async_trait]
impl ToolExecutor for SnapshotTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let selector = call
            .arguments
            .get("selector")
            .and_then(|v| v.as_str())
            .unwrap_or("body");

        #[cfg(feature = "browser")]
        {
            let state = self.state.lock().await;
            let tab = match state.tab() {
                Ok(t) => t,
                Err(e) => return ToolResult::error(&call, e),
            };
            let js = format!(
                "(function() {{ var el = document.querySelector({selector:?}); if (!el) return 'Element not found: {selector}'; return el.innerText; }})()",
            );
            let result = tab
                .evaluate(&js, false)
                .map_err(|e| format!("snapshot failed: {e}"))
                .and_then(|v| {
                    v.value
                        .and_then(|v| v.as_str().map(String::from))
                        .ok_or_else(|| "empty snapshot result".to_string())
                });
            match result {
                Ok(text) => {
                    let truncated = if text.len() > 50000 {
                        format!(
                            "{}...\n(truncated, {} chars total)",
                            &text[..50000],
                            text.len()
                        )
                    } else {
                        text
                    };
                    ToolResult::success(&call, truncated)
                }
                Err(e) => ToolResult::error(&call, e),
            }
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = selector;
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
        super::super::make_shared_state(PathBuf::from("/tmp/zstar-test/screenshots"))
    }

    #[test]
    fn descriptor_name() {
        let tool = SnapshotTool::new(make_state());
        assert_eq!(tool.descriptor().name, "browser_snapshot");
        assert_eq!(tool.descriptor().parameters.len(), 1);
        assert!(!tool.descriptor().parameters[0].required);
    }
}
