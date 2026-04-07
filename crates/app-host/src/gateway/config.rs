use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::paths;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayConfig {
    #[serde(default)]
    pub platforms: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_message_length: Option<usize>,
}

impl GatewayConfig {
    pub fn load_or_default(home: impl AsRef<Path>) -> Self {
        let path = config_path(home);
        if !path.exists() {
            return Self::default();
        }
        let body = match fs::read_to_string(&path) {
            Ok(body) => body,
            Err(_) => return Self::default(),
        };
        let interpolated = interpolate_env_vars(&body);
        serde_json::from_str(&interpolated).unwrap_or_default()
    }

    pub fn resolve_model(&self) -> String {
        self.model
            .clone()
            .or_else(|| env::var("MATRIXCLAW_LLM_MODEL").ok())
            .unwrap_or_else(|| "moonshotai/kimi-k2.5".to_string())
    }
}

fn config_path(home: impl AsRef<Path>) -> std::path::PathBuf {
    paths::config_dir(home).join("gateway.json")
}

fn interpolate_env_vars(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek() == Some(&'{') {
            chars.next();
            let mut var_name = String::new();
            loop {
                match chars.next() {
                    Some('}') => break,
                    Some(c) => var_name.push(c),
                    None => {
                        result.push_str("${");
                        result.push_str(&var_name);
                        break;
                    }
                }
            }
            if let Ok(value) = env::var(&var_name) {
                result.push_str(&value);
            } else {
                result.push_str(&format!("${{{var_name}}}"));
            }
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_from_json() {
        let json = r#"{
            "platforms": {
                "matrix": {
                    "homeserver": "https://matrix.org",
                    "access_token": "test-token"
                }
            },
            "model": "test-model",
            "max_message_length": 4000
        }"#;
        let config: GatewayConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.model.as_deref(), Some("test-model"));
        assert_eq!(config.max_message_length, Some(4000));
        assert!(config.platforms.contains_key("matrix"));
    }

    #[test]
    fn config_minimal() {
        let json = r#"{"platforms":{}}"#;
        let config: GatewayConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.model, None);
        assert_eq!(config.max_message_length, None);
    }

    #[test]
    fn env_var_interpolation() {
        env::set_var("TEST_GATEWAY_TOKEN", "secret-123");
        let input = r#"{"platforms":{"matrix":{"token":"${TEST_GATEWAY_TOKEN}"}}}"#;
        let result = interpolate_env_vars(input);
        env::remove_var("TEST_GATEWAY_TOKEN");
        assert!(result.contains("secret-123"));
        assert!(!result.contains("${TEST_GATEWAY_TOKEN}"));
    }

    #[test]
    fn env_var_missing_keeps_placeholder() {
        let input = r#"{"token":"${NONEXISTENT_VAR_XYZ}"}"#;
        let result = interpolate_env_vars(input);
        assert!(result.contains("${NONEXISTENT_VAR_XYZ}"));
    }

    #[test]
    fn resolve_model_uses_config_first() {
        let config = GatewayConfig {
            platforms: HashMap::new(),
            model: Some("my-model".to_string()),
            max_message_length: None,
        };
        assert_eq!(config.resolve_model(), "my-model");
    }

    #[test]
    fn resolve_model_falls_back_to_default() {
        env::remove_var("MATRIXCLAW_LLM_MODEL");
        let config = GatewayConfig::default();
        assert_eq!(config.resolve_model(), "moonshotai/kimi-k2.5");
    }
}
