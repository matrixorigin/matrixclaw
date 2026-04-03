pub mod calculator;
pub mod delegate;
pub mod environment;
pub mod filesystem;
pub mod memory;
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
    registry.register(Arc::new(memory::MemoryTool::new())).await;
    registry
        .register(Arc::new(stubs::CodeInterpreterTool::new()))
        .await;
    registry.register(Arc::new(stubs::SkillsTool::new())).await;
}
