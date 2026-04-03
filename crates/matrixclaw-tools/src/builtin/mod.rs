pub mod calculator;
pub mod delegate;
pub mod environment;
pub mod filesystem;
pub mod memory;
pub mod skills;
pub mod stubs;
pub mod terminal;
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
    registry
        .register(Arc::new(stubs::CodeInterpreterTool::new()))
        .await;
    registry
        .register(Arc::new(skills::SkillsTool::new(std::path::Path::new(
            workspace_root,
        ))))
        .await;
}
