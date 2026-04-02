use std::fs;
use std::path::Path;

use crate::backend::ProviderPlaneConfig;

pub fn load_provider_config(config_path: &Path) -> Result<ProviderPlaneConfig, String> {
    let content = fs::read_to_string(config_path)
        .map_err(|e| format!("failed to read provider config: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("failed to parse provider config: {e}"))
}

pub fn load_or_default_config(config_path: Option<&Path>) -> ProviderPlaneConfig {
    let Some(path) = config_path else {
        return ProviderPlaneConfig {
            providers: vec![],
            fallback_chain: vec![],
        };
    };
    load_provider_config(path).unwrap_or_else(|e| {
        eprintln!("warning: {e}, using empty provider config");
        ProviderPlaneConfig {
            providers: vec![],
            fallback_chain: vec![],
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::ProviderType;

    #[test]
    fn loads_provider_plane_config() {
        let json = r#"{
            "providers": [
                {
                    "name": "openrouter",
                    "type": "open_ai",
                    "api_key": "sk-or-test",
                    "models": ["moonshotai/kimi-k2.5"]
                },
                {
                    "name": "local",
                    "type": "ollama",
                    "models": ["llama3"]
                }
            ],
            "fallback_chain": ["openrouter", "local"]
        }"#;
        let config: ProviderPlaneConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.providers.len(), 2);
        assert_eq!(config.fallback_chain, vec!["openrouter", "local"]);
        assert_eq!(config.providers[0].provider_type, ProviderType::OpenAi);
        assert_eq!(config.providers[1].provider_type, ProviderType::Ollama);
    }

    #[test]
    fn empty_config_uses_defaults() {
        let config = load_or_default_config(None);
        assert!(config.providers.is_empty());
        assert!(config.fallback_chain.is_empty());
    }

    #[test]
    fn missing_file_uses_defaults() {
        let config = load_or_default_config(Some(Path::new("/nonexistent/providers.json")));
        assert!(config.providers.is_empty());
    }
}
