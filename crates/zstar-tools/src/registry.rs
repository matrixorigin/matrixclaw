use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::descriptor::ToolDescriptor;
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct ToolRegistry {
    tools: RwLock<HashMap<String, Arc<dyn ToolExecutor>>>,
}

impl std::fmt::Debug for ToolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRegistry").finish_non_exhaustive()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, tool: Arc<dyn ToolExecutor>) {
        let name = tool.descriptor().name.clone();
        self.tools.write().await.insert(name, tool);
    }

    pub async fn get(&self, name: &str) -> Option<Arc<dyn ToolExecutor>> {
        self.tools.read().await.get(name).cloned()
    }

    pub async fn list_descriptors(&self) -> Vec<ToolDescriptor> {
        self.tools
            .read()
            .await
            .values()
            .map(|t| t.descriptor().clone())
            .collect()
    }

    pub async fn execute(&self, call: ToolCall) -> ToolResult {
        let tool = self.get(&call.name).await;
        match tool {
            Some(t) => t.execute(call).await,
            None => ToolResult::error(&call, format!("unknown tool: {}", call.name)),
        }
    }

    pub async fn has(&self, name: &str) -> bool {
        self.tools.read().await.contains_key(name)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
