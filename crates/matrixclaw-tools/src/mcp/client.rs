use std::sync::Arc;

use super::transport::StdioTransport;
use super::types::{CallToolResult, InitializeResult, ListToolsResult, McpTool};

pub struct McpClient {
    transport: Arc<StdioTransport>,
    server_info: Option<ServerInfoOwned>,
}

struct ServerInfoOwned {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    version: String,
}

impl std::fmt::Debug for McpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpClient").finish_non_exhaustive()
    }
}

impl McpClient {
    pub async fn connect(
        command: &str,
        args: &[String],
        env: &[(String, String)],
    ) -> Result<Self, String> {
        let transport = StdioTransport::spawn(command, args, env).await?;
        let client = Self {
            transport: Arc::new(transport),
            server_info: None,
        };
        Ok(client)
    }

    pub async fn initialize(mut self) -> Result<Self, String> {
        let params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "matrixclaw",
                "version": "0.1.0"
            }
        });

        let response = self
            .transport
            .send_request("initialize", Some(params))
            .await?;
        if let Some(error) = &response.error {
            return Err(format!(
                "MCP initialize error (code {}): {}",
                error.code, error.message
            ));
        }

        let result: InitializeResult =
            serde_json::from_value(response.result.ok_or("MCP initialize returned no result")?)
                .map_err(|e| format!("failed to parse initialize result: {e}"))?;

        self.server_info = Some(ServerInfoOwned {
            name: result.server_info.name,
            version: result.server_info.version,
        });

        let _ = self
            .transport
            .send_notification("notifications/initialized", Some(serde_json::json!({})))
            .await;

        Ok(self)
    }

    pub async fn list_tools(&self) -> Result<Vec<McpTool>, String> {
        let response = self
            .transport
            .send_request("tools/list", Some(serde_json::json!({})))
            .await?;

        if let Some(error) = &response.error {
            return Err(format!(
                "MCP tools/list error (code {}): {}",
                error.code, error.message
            ));
        }

        let result: ListToolsResult =
            serde_json::from_value(response.result.ok_or("MCP tools/list returned no result")?)
                .map_err(|e| format!("failed to parse tools/list result: {e}"))?;

        Ok(result.tools)
    }

    pub async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<CallToolResult, String> {
        let params = serde_json::json!({
            "name": name,
            "arguments": arguments
        });

        let response = self
            .transport
            .send_request("tools/call", Some(params))
            .await?;

        if let Some(error) = &response.error {
            return Err(format!(
                "MCP tools/call error (code {}): {}",
                error.code, error.message
            ));
        }

        let result: CallToolResult =
            serde_json::from_value(response.result.ok_or("MCP tools/call returned no result")?)
                .map_err(|e| format!("failed to parse tools/call result: {e}"))?;

        Ok(result)
    }

    pub fn transport(&self) -> Arc<StdioTransport> {
        Arc::clone(&self.transport)
    }
}
