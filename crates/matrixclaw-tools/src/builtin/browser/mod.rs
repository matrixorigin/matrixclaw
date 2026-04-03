pub mod click;
pub mod close;
pub mod get_url;
pub mod go_back;
pub mod navigate;
pub mod screenshot;
pub mod scroll;
pub mod snapshot;
pub mod r#type;

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::registry::ToolRegistry;

pub struct BrowserState {
    #[cfg(feature = "browser")]
    browser: Option<headless_chrome::Browser>,
    #[cfg(feature = "browser")]
    tab: Option<Arc<headless_chrome::Tab>>,
    screenshots_dir: PathBuf,
}

impl BrowserState {
    pub fn new(screenshots_dir: PathBuf) -> Self {
        Self {
            #[cfg(feature = "browser")]
            browser: None,
            #[cfg(feature = "browser")]
            tab: None,
            screenshots_dir,
        }
    }

    #[cfg(feature = "browser")]
    pub async fn ensure_browser(&mut self) -> Result<(), String> {
        if self.browser.is_none() {
            std::fs::create_dir_all(&self.screenshots_dir)
                .map_err(|e| format!("failed to create screenshots dir: {e}"))?;
            let browser = headless_chrome::Browser::new(headless_chrome::LaunchOptions {
                headless: true,
                sandbox: false,
                window_size: Some((1920, 1080)),
                args: vec![
                    "--no-sandbox".into(),
                    "--disable-gpu".into(),
                    "--disable-dev-shm-usage".into(),
                ],
                ..Default::default()
            })
            .map_err(|e| format!("failed to launch browser: {e}"))?;
            let tab = browser
                .new_tab()
                .map_err(|e| format!("failed to create tab: {e}"))?;
            self.browser = Some(browser);
            self.tab = Some(tab);
        }
        Ok(())
    }

    #[cfg(feature = "browser")]
    pub fn tab(&self) -> Result<&Arc<headless_chrome::Tab>, String> {
        self.tab
            .as_ref()
            .ok_or_else(|| "browser not initialized".to_string())
    }

    pub fn screenshots_dir(&self) -> &PathBuf {
        &self.screenshots_dir
    }

    #[cfg(feature = "browser")]
    pub fn close(&mut self) {
        self.tab = None;
        self.browser = None;
    }
}

pub type SharedBrowserState = Arc<Mutex<BrowserState>>;

pub fn make_shared_state(screenshots_dir: PathBuf) -> SharedBrowserState {
    Arc::new(Mutex::new(BrowserState::new(screenshots_dir)))
}

pub async fn register_all(registry: &ToolRegistry, state: SharedBrowserState) {
    registry
        .register(Arc::new(navigate::NavigateTool::new(state.clone())))
        .await;
    registry
        .register(Arc::new(snapshot::SnapshotTool::new(state.clone())))
        .await;
    registry
        .register(Arc::new(click::ClickTool::new(state.clone())))
        .await;
    registry
        .register(Arc::new(r#type::TypeTool::new(state.clone())))
        .await;
    registry
        .register(Arc::new(scroll::ScrollTool::new(state.clone())))
        .await;
    registry
        .register(Arc::new(go_back::GoBackTool::new(state.clone())))
        .await;
    registry
        .register(Arc::new(get_url::GetUrlTool::new(state.clone())))
        .await;
    registry
        .register(Arc::new(screenshot::ScreenshotTool::new(state.clone())))
        .await;
    registry
        .register(Arc::new(close::CloseTool::new(state)))
        .await;
}
