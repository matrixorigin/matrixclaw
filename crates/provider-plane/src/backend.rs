use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: ProviderType,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default)]
    pub rpm_limit: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    OpenAi,
    Anthropic,
    Ollama,
    Custom,
}

impl ProviderConfig {
    pub fn resolve_base_url(&self) -> &str {
        self.base_url
            .as_deref()
            .unwrap_or_else(|| match self.provider_type {
                ProviderType::OpenAi => "https://openrouter.ai/api/v1",
                ProviderType::Anthropic => "https://api.anthropic.com/v1",
                ProviderType::Ollama => "http://localhost:11434",
                ProviderType::Custom => "http://localhost:8080",
            })
    }

    pub fn api_key_or_empty(&self) -> &str {
        self.api_key.as_deref().unwrap_or("")
    }

    pub fn effective_model(&self, fallback: &str) -> String {
        self.models
            .first()
            .cloned()
            .unwrap_or_else(|| fallback.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPlaneConfig {
    pub providers: Vec<ProviderConfig>,
    #[serde(default)]
    pub fallback_chain: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_provider_config_with_defaults() {
        let config: ProviderConfig = serde_json::from_str(
            r#"{
            "name": "openrouter",
            "type": "open_ai",
            "api_key": "sk-test"
        }"#,
        )
        .unwrap();
        assert_eq!(config.name, "openrouter");
        assert_eq!(config.provider_type, ProviderType::OpenAi);
        assert!(config.base_url.is_none());
        assert_eq!(config.rpm_limit, None);
        assert_eq!(config.resolve_base_url(), "https://openrouter.ai/api/v1");
    }

    #[test]
    fn parses_fallback_chain() {
        let plane: ProviderPlaneConfig = serde_json::from_str(
            r#"{
            "providers": [
                {"name": "primary", "type": "open_ai", "api_key": "sk-1"},
                {"name": "fallback", "type": "ollama"}
            ],
            "fallback_chain": ["primary", "fallback"]
        }"#,
        )
        .unwrap();
        assert_eq!(plane.fallback_chain, vec!["primary", "fallback"]);
    }

    #[test]
    fn effective_model_uses_first_configured() {
        let config = ProviderConfig {
            name: "test".to_string(),
            provider_type: ProviderType::OpenAi,
            base_url: None,
            api_key: None,
            models: vec!["gpt-4o".to_string(), "gpt-3.5-turbo".to_string()],
            rpm_limit: None,
        };
        assert_eq!(config.effective_model("default"), "gpt-4o");
    }
}
