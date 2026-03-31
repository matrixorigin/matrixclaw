use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: &'static str,
    pub id: u64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    pub fn new(id: u64, method: &str, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            method: method.to_string(),
            params,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcResponse {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: Option<u64>,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[allow(dead_code)]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    #[allow(dead_code)]
    pub protocol_version: String,
    #[allow(dead_code)]
    pub capabilities: Value,
    #[allow(dead_code)]
    pub server_info: ServerInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListToolsResult {
    pub tools: Vec<McpTool>,
    #[allow(dead_code)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpTool {
    pub name: String,
    #[allow(dead_code)]
    pub description: Option<String>,
    #[allow(dead_code)]
    pub input_schema: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallToolResult {
    pub content: Vec<ToolContent>,
    #[allow(dead_code)]
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        #[allow(dead_code)]
        data: String,
        #[allow(dead_code)]
        mime_type: String,
    },
    #[serde(rename = "resource")]
    Resource {
        #[allow(dead_code)]
        resource: Value,
    },
}

impl CallToolResult {
    pub fn text_output(&self) -> String {
        let mut out = String::new();
        for content in &self.content {
            match content {
                ToolContent::Text { text } => {
                    if !out.is_empty() {
                        out.push('\n');
                    }
                    out.push_str(text);
                }
                _ => {}
            }
        }
        out
    }

    pub fn is_error(&self) -> bool {
        self.is_error.unwrap_or(false)
    }
}
