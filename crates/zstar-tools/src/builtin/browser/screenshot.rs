use async_trait::async_trait;

use super::SharedBrowserState;
use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct ScreenshotTool {
    descriptor: ToolDescriptor,
    state: SharedBrowserState,
}

impl ScreenshotTool {
    pub fn new(state: SharedBrowserState) -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "browser_screenshot",
                "Take a PNG screenshot of the current page and save to disk",
            )
            .with_parameters(vec![ToolParameter::optional(
                "filename",
                ParameterType::String,
                "Custom filename for the screenshot (auto-generated if omitted)",
            )]),
            state,
        }
    }
}

#[async_trait]
impl ToolExecutor for ScreenshotTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let custom_name = call
            .arguments
            .get("filename")
            .and_then(|v| v.as_str())
            .map(String::from);

        #[cfg(feature = "browser")]
        {
            let state = self.state.lock().await;
            let dir = state.screenshots_dir().clone();
            let tab = match state.tab() {
                Ok(t) => t,
                Err(e) => return ToolResult::error(&call, e),
            };
            std::fs::create_dir_all(&dir)
                .map_err(|e| format!("failed to create screenshots dir: {e}"))
                .ok();
            let filename = custom_name.unwrap_or_else(|| {
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                format!("screenshot_{ts}.png")
            });
            let path = dir.join(&filename);
            let png_data = match tab.capture_screenshot(
                headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
                None,
                None,
                true,
            ) {
                Ok(data) => data,
                Err(e) => {
                    return ToolResult::error(&call, format!("screenshot capture failed: {e}"))
                }
            };
            if let Err(e) = std::fs::write(&path, png_data) {
                return ToolResult::error(&call, format!("failed to write screenshot: {e}"));
            }
            ToolResult::success(&call, format!("Screenshot saved: {}", path.display()))
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = custom_name;
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
        let tool = ScreenshotTool::new(make_state());
        assert_eq!(tool.descriptor().name, "browser_screenshot");
        assert_eq!(tool.descriptor().parameters.len(), 1);
        assert!(!tool.descriptor().parameters[0].required);
    }
}
