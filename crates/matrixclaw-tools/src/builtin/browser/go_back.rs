use async_trait::async_trait;

use super::SharedBrowserState;
use crate::descriptor::ToolDescriptor;
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct GoBackTool {
    descriptor: ToolDescriptor,
    state: SharedBrowserState,
}

impl GoBackTool {
    pub fn new(state: SharedBrowserState) -> Self {
        Self {
            descriptor: ToolDescriptor::new("browser_go_back", "Navigate back in browser history"),
            state,
        }
    }
}

#[async_trait]
impl ToolExecutor for GoBackTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        #[cfg(feature = "browser")]
        {
            let state = self.state.lock().await;
            let tab = match state.tab() {
                Ok(t) => t,
                Err(e) => return ToolResult::error(&call, e),
            };
            let js = "window.history.back()";
            if let Err(e) = tab.evaluate(js, false) {
                return ToolResult::error(&call, format!("go back failed: {e}"));
            }
            std::thread::sleep(std::time::Duration::from_millis(1000));
            let url = tab.get_url().unwrap_or_else(|_| "unknown".to_string());
            ToolResult::success(&call, format!("Navigated back to: {url}"))
        }

        #[cfg(not(feature = "browser"))]
        {
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

    #[test]
    fn descriptor_name() {
        let state =
            super::super::make_shared_state(PathBuf::from("/tmp/matrixclaw-test/screenshots"));
        let tool = GoBackTool::new(state);
        assert_eq!(tool.descriptor().name, "browser_go_back");
        assert!(tool.descriptor().parameters.is_empty());
    }
}
