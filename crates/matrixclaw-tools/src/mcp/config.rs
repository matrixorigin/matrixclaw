use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    #[serde(default)]
    pub servers: HashMap<String, McpServerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub disabled: bool,
}

impl McpConfig {
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content =
            fs::read_to_string(path).map_err(|e| format!("failed to read MCP config: {e}"))?;
        serde_json::from_str(&content).map_err(|e| format!("failed to parse MCP config: {e}"))
    }

    pub fn active_servers(&self) -> Vec<(&str, &McpServerConfig)> {
        self.servers
            .iter()
            .filter(|(_, config)| !config.disabled)
            .map(|(name, config)| (name.as_str(), config))
            .collect()
    }
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_empty_config() {
        let config: McpConfig = serde_json::from_str("{}").unwrap();
        assert!(config.servers.is_empty());
    }

    #[test]
    fn parses_server_config() {
        let config: McpConfig = serde_json::from_str(
            r#"{
            "servers": {
                "filesystem": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
                    "env": {}
                }
            }
        }"#,
        )
        .unwrap();

        assert_eq!(config.servers.len(), 1);
        let fs_server = &config.servers["filesystem"];
        assert_eq!(fs_server.command, "npx");
        assert_eq!(fs_server.args.len(), 3);
        assert!(!fs_server.disabled);
    }

    #[test]
    fn active_servers_excludes_disabled() {
        let mut config = McpConfig::default();
        config.servers.insert(
            "active".to_string(),
            McpServerConfig {
                command: "echo".to_string(),
                args: vec![],
                env: HashMap::new(),
                disabled: false,
            },
        );
        config.servers.insert(
            "disabled".to_string(),
            McpServerConfig {
                command: "echo".to_string(),
                args: vec![],
                env: HashMap::new(),
                disabled: true,
            },
        );

        let active = config.active_servers();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].0, "active");
    }
}
