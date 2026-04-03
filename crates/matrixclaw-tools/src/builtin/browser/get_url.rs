use async_trait::async_trait;

use super::SharedBrowserState;
use crate::descriptor::ToolDescriptor;
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct GetUrlTool {
    descriptor: ToolDescriptor,
    state: SharedBrowserState,
}

impl GetUrlTool {
    pub fn new(state: SharedBrowserState) -> Self {
        Self {
            descriptor: ToolDescriptor::new("browser_get_url", "Get the current page URL"),
            state,
        }
    }
}

#[async_trait]
impl ToolExecutor for GetUrlTool {
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
            match tab.get_url() {
                Ok(url) => ToolResult::success(&call, url),
                Err(e) => ToolResult::error(&call, format!("failed to get URL: {e}")),
            }
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
        let tool = GetUrlTool::new(state);
        assert_eq!(tool.descriptor().name, "browser_get_url");
        assert!(tool.descriptor().parameters.is_empty());
    }
}
