use async_trait::async_trait;

use super::SharedBrowserState;
use crate::descriptor::ToolDescriptor;
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct CloseTool {
    descriptor: ToolDescriptor,
    state: SharedBrowserState,
}

impl CloseTool {
    pub fn new(state: SharedBrowserState) -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "browser_close",
                "Close the headless browser and release all resources",
            ),
            state,
        }
    }
}

#[async_trait]
impl ToolExecutor for CloseTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        #[cfg(feature = "browser")]
        {
            let mut state = self.state.lock().await;
            state.close();
            ToolResult::success(&call, "Browser closed")
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
        let tool = CloseTool::new(state);
        assert_eq!(tool.descriptor().name, "browser_close");
        assert!(tool.descriptor().parameters.is_empty());
    }
}
