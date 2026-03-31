use std::path::Path;
use std::sync::Arc;

use crate::registry::ToolRegistry;

use super::adapter::McpToolAdapter;
use super::client::McpClient;
use super::config::McpConfig;

#[derive(Debug)]
pub struct McpRegistrationReport {
    pub servers_connected: Vec<String>,
    pub servers_failed: Vec<(String, String)>,
    pub tools_registered: Vec<String>,
}

pub async fn register_mcp_tools(
    registry: &ToolRegistry,
    config_path: &Path,
) -> McpRegistrationReport {
    let config = match McpConfig::load(config_path) {
        Ok(c) => c,
        Err(e) => {
            return McpRegistrationReport {
                servers_connected: Vec::new(),
                servers_failed: vec![("config".to_string(), e)],
                tools_registered: Vec::new(),
            }
        }
    };

    let mut report = McpRegistrationReport {
        servers_connected: Vec::new(),
        servers_failed: Vec::new(),
        tools_registered: Vec::new(),
    };

    for (name, server_config) in config.active_servers() {
        let env_pairs: Vec<(String, String)> = server_config
            .env
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let client = match McpClient::connect(&server_config.command, &server_config.args, &env_pairs).await {
            Ok(c) => c,
            Err(e) => {
                report.servers_failed.push((name.to_string(), e));
                continue;
            }
        };

        let client = match client.initialize().await {
            Ok(c) => Arc::new(c),
            Err(e) => {
                report.servers_failed.push((name.to_string(), e));
                continue;
            }
        };

        let tools = match client.list_tools().await {
            Ok(t) => t,
            Err(e) => {
                report.servers_failed.push((name.to_string(), e));
                continue;
            }
        };

        report.servers_connected.push(name.to_string());

        for tool in &tools {
            let tool_name = format!("mcp__{}__{}", name, tool.name);
            let mut mcp_tool = tool.clone();
            mcp_tool.name = tool_name;
            let adapter = McpToolAdapter::new(&mcp_tool, Arc::clone(&client));
            report.tools_registered.push(mcp_tool.name.clone());
            registry.register(Arc::new(adapter)).await;
        }
    }

    report
}
