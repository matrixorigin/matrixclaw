#[cfg(feature = "browser")]
pub mod browser;
pub mod calculator;
pub mod clarify;
pub mod code_interpreter;
pub mod cronjob;
pub mod delegate;
pub mod environment;
pub mod filesystem;
pub mod memory;
pub mod process;
pub mod search_files;
pub mod session_search;
pub mod skills;
pub mod terminal;
pub mod todo;
pub mod web;

use std::sync::Arc;

use crate::registry::ToolRegistry;

pub async fn register_all(registry: &ToolRegistry, workspace_root: &str) {
    registry
        .register(Arc::new(terminal::TerminalTool::new(workspace_root)))
        .await;
    registry
        .register(Arc::new(filesystem::ReadFileTool::new(workspace_root)))
        .await;
    registry
        .register(Arc::new(filesystem::WriteFileTool::new(workspace_root)))
        .await;
    registry
        .register(Arc::new(filesystem::ListDirectoryTool::new(workspace_root)))
        .await;
    registry
        .register(Arc::new(filesystem::EditFileTool::new(workspace_root)))
        .await;
    registry.register(Arc::new(web::WebFetchTool::new())).await;
    registry.register(Arc::new(web::WebSearchTool::new())).await;
    registry
        .register(Arc::new(calculator::CalculatorTool::new()))
        .await;
    registry
        .register(Arc::new(environment::EnvironmentTool::new()))
        .await;
    let memory_db_path = memory::MemoryTool::db_path_for_home(std::path::Path::new(workspace_root));
    match memory::MemoryTool::open(&memory_db_path) {
        Ok(tool) => {
            registry.register(Arc::new(tool)).await;
        }
        Err(e) => eprintln!("warning: failed to open memory store: {e}"),
    }
    let session_db_path =
        session_search::SessionSearchTool::db_path_for_home(std::path::Path::new(workspace_root));
    if session_db_path.exists() {
        match session_search::SessionSearchTool::open(&session_db_path) {
            Ok(tool) => {
                registry.register(Arc::new(tool)).await;
            }
            Err(e) => eprintln!("warning: failed to open session search: {e}"),
        }
    }
    registry
        .register(Arc::new(code_interpreter::CodeInterpreterTool::new()))
        .await;
    registry
        .register(Arc::new(skills::SkillsTool::new(std::path::Path::new(
            workspace_root,
        ))))
        .await;
    registry.register(Arc::new(todo::TodoTool::new())).await;
    registry
        .register(Arc::new(search_files::SearchFilesTool::new(workspace_root)))
        .await;
    registry
        .register(Arc::new(clarify::ClarifyTool::new()))
        .await;
    registry
        .register(Arc::new(process::ProcessTool::new()))
        .await;
    let cron_db_path = cronjob::CronjobTool::db_path_for_home(std::path::Path::new(workspace_root));
    match cronjob::CronjobTool::open(&cron_db_path) {
        Ok(tool) => {
            registry.register(Arc::new(tool)).await;
        }
        Err(e) => eprintln!("warning: failed to open cron store: {e}"),
    }
    #[cfg(feature = "browser")]
    {
        let screenshots_dir =
            std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()))
                .join(".matrixclaw")
                .join("screenshots");
        let state = browser::make_shared_state(screenshots_dir);
        browser::register_all(registry, state).await;
    }
}
