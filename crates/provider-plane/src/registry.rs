use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::backend::{ProviderConfig, ProviderType};
use crate::openai::OpenAiProvider;
use zstar_agent_core::provider::Provider;

#[derive(Clone)]
pub struct ProviderEntry {
    name: String,
    backend: Arc<RwLock<Box<dyn Provider>>>,
}

impl ProviderEntry {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn provider_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, Box<dyn Provider>> {
        self.backend.write().await
    }
}

pub struct ProviderRegistry {
    entries: RwLock<HashMap<String, ProviderEntry>>,
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, config: ProviderConfig) -> Result<(), String> {
        let provider: Box<dyn Provider> = match config.provider_type {
            ProviderType::OpenAi | ProviderType::Custom => {
                let base_url = config.resolve_base_url().to_string();
                let api_key = config.api_key_or_empty().to_string();
                let model = config.effective_model("gpt-4o");
                Box::new(
                    OpenAiProvider::with_base_url(&base_url, &api_key, &model).map_err(|e| e.0)?,
                )
            }
            ProviderType::Ollama => {
                let base_url = config.resolve_base_url().to_string();
                let model = config.effective_model("llama3");
                Box::new(
                    OpenAiProvider::with_base_url(format!("{base_url}/v1"), "", &model)
                        .map_err(|e| e.0)?,
                )
            }
            ProviderType::Anthropic => {
                let api_key = config.api_key_or_empty().to_string();
                let model = config.effective_model("claude-sonnet-4-20250514");
                Box::new(
                    OpenAiProvider::with_base_url("https://openrouter.ai/api/v1", &api_key, &model)
                        .map_err(|e| e.0)?,
                )
            }
        };

        let name = config.name.clone();
        let entry = ProviderEntry {
            name: name.clone(),
            backend: Arc::new(RwLock::new(provider)),
        };

        let mut entries = self.entries.write().await;
        entries.insert(name, entry);
        Ok(())
    }

    pub async fn get(&self, name: &str) -> Option<ProviderEntry> {
        let entries = self.entries.read().await;
        entries.get(name).cloned()
    }

    pub async fn provider_names(&self) -> Vec<String> {
        let entries = self.entries.read().await;
        entries.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::ProviderType;

    #[tokio::test]
    async fn registers_and_looks_up_provider() {
        let registry = ProviderRegistry::new();
        let config = ProviderConfig {
            name: "test-openai".to_string(),
            provider_type: ProviderType::OpenAi,
            base_url: None,
            api_key: Some("sk-test".to_string()),
            models: vec!["gpt-4o".to_string()],
            rpm_limit: None,
        };
        registry.register(config).await.unwrap();
        let backend = registry.get("test-openai").await.unwrap();
        assert_eq!(backend.name(), "test-openai");
    }

    #[tokio::test]
    async fn returns_none_for_unknown_provider() {
        let registry = ProviderRegistry::new();
        assert!(registry.get("nonexistent").await.is_none());
    }

    #[tokio::test]
    async fn lists_provider_names() {
        let registry = ProviderRegistry::new();
        let config = ProviderConfig {
            name: "a".to_string(),
            provider_type: ProviderType::Ollama,
            base_url: None,
            api_key: None,
            models: vec![],
            rpm_limit: None,
        };
        registry.register(config).await.unwrap();
        let names = registry.provider_names().await;
        assert_eq!(names, vec!["a"]);
    }
}
