use std::io::{BufRead, Write};
use std::sync::Arc;

use serde::Deserialize;
use serde_json::{json, Value};

use matrixclaw_tools::ToolRegistry;

pub struct McpServer {
    registry: Arc<ToolRegistry>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Value,
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

#[derive(Debug)]
struct JsonRpcError {
    code: i32,
    message: String,
}

impl JsonRpcResponse {
    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert("jsonrpc".into(), Value::String(self.jsonrpc.to_string()));
        map.insert("id".into(), self.id.clone());
        if let Some(ref result) = self.result {
            map.insert("result".into(), result.clone());
        }
        if let Some(ref error) = self.error {
            let mut err = serde_json::Map::new();
            err.insert("code".into(), json!(error.code));
            err.insert("message".into(), Value::String(error.message.clone()));
            map.insert("error".into(), Value::Object(err));
        }
        Value::Object(map)
    }
}

impl McpServer {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self { registry }
    }

    pub async fn run(&self) -> Result<(), String> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => return Err(format!("stdin read error: {e}")),
            };

            if line.trim().is_empty() {
                continue;
            }

            let request: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(e) => {
                    let resp = JsonRpcResponse {
                        jsonrpc: "2.0",
                        id: Value::Null,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32700,
                            message: format!("parse error: {e}"),
                        }),
                    };
                    let _ = writeln!(stdout, "{}", resp.to_json());
                    let _ = stdout.flush();
                    continue;
                }
            };

            let id = request.id.unwrap_or(Value::Null);
            let result = self.handle_method(&request.method, request.params).await;

            let response = JsonRpcResponse {
                jsonrpc: "2.0",
                id,
                result: Some(result),
                error: None,
            };

            let _ = writeln!(stdout, "{}", response.to_json());
            let _ = stdout.flush();
        }

        Ok(())
    }

    async fn handle_method(&self, method: &str, params: Option<Value>) -> Value {
        match method {
            "initialize" => json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "matrixclaw",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
            "notifications/initialized" => json!({}),
            "tools/list" => {
                let descriptors = self.registry.list_descriptors().await;
                let tools: Vec<Value> = descriptors
                    .iter()
                    .map(|d| {
                        let mut properties = serde_json::Map::new();
                        let mut required = Vec::new();
                        for param in &d.parameters {
                            let mut prop = serde_json::Map::new();
                            prop.insert(
                                "type".into(),
                                Value::String(param.param_type.to_json_type().to_string()),
                            );
                            prop.insert(
                                "description".into(),
                                Value::String(param.description.clone()),
                            );
                            if let Some(ref enum_vals) = param.enum_values {
                                prop.insert(
                                    "enum".into(),
                                    Value::Array(
                                        enum_vals
                                            .iter()
                                            .map(|v| Value::String(v.clone()))
                                            .collect(),
                                    ),
                                );
                            }
                            properties.insert(param.name.clone(), Value::Object(prop));
                            if param.required {
                                required.push(Value::String(param.name.clone()));
                            }
                        }
                        json!({
                            "name": d.name,
                            "description": d.description,
                            "inputSchema": {
                                "type": "object",
                                "properties": properties,
                                "required": required,
                            }
                        })
                    })
                    .collect();
                json!({"tools": tools})
            }
            "tools/call" => {
                let params = match params {
                    Some(p) => p,
                    None => {
                        return json!({
                            "content": [{"type": "text", "text": "missing params"}],
                            "isError": true
                        })
                    }
                };
                let tool_name = match params.get("name").and_then(|v| v.as_str()) {
                    Some(n) => n,
                    None => {
                        return json!({
                            "content": [{"type": "text", "text": "missing tool name"}],
                            "isError": true
                        })
                    }
                };
                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
                let call = matrixclaw_tools::ToolCall::new(
                    "mcp".to_string(),
                    tool_name.to_string(),
                    arguments,
                );
                let result = self.registry.execute(call).await;
                json!({
                    "content": [{"type": "text", "text": result.output}],
                    "isError": result.is_error
                })
            }
            "resources/list" => json!({"resources": []}),
            "prompts/list" => json!({"prompts": []}),
            _ => json!({
                "error": {
                    "code": -32601,
                    "message": format!("method not found: {method}")
                }
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn handle_initialize() {
        let registry = Arc::new(ToolRegistry::new());
        let server = McpServer::new(registry);
        let result = server.handle_method("initialize", None).await;
        assert_eq!(result["serverInfo"]["name"], "matrixclaw");
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert!(result.get("capabilities").is_some());
    }

    #[tokio::test]
    async fn handle_tools_list_empty() {
        let registry = Arc::new(ToolRegistry::new());
        let server = McpServer::new(registry);
        let result = server.handle_method("tools/list", None).await;
        assert!(result["tools"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn handle_unknown_method() {
        let registry = Arc::new(ToolRegistry::new());
        let server = McpServer::new(registry);
        let result = server.handle_method("nonexistent", None).await;
        assert!(result.get("error").is_some());
        assert_eq!(result["error"]["code"], -32601);
    }

    #[tokio::test]
    async fn handle_tools_call_missing_name() {
        let registry = Arc::new(ToolRegistry::new());
        let server = McpServer::new(registry);
        let result = server
            .handle_method("tools/call", Some(json!({"arguments": {}})))
            .await;
        assert_eq!(result["isError"], true);
    }

    #[tokio::test]
    async fn handle_tools_call_unknown_tool() {
        let registry = Arc::new(ToolRegistry::new());
        let server = McpServer::new(registry);
        let result = server
            .handle_method(
                "tools/call",
                Some(json!({"name": "nonexistent_tool", "arguments": {}})),
            )
            .await;
        assert_eq!(result["isError"], true);
    }

    #[tokio::test]
    async fn handle_notifications_initialized() {
        let registry = Arc::new(ToolRegistry::new());
        let server = McpServer::new(registry);
        let result = server
            .handle_method("notifications/initialized", None)
            .await;
        assert_eq!(result, json!({}));
    }

    #[tokio::test]
    async fn handle_resources_and_prompts_list() {
        let registry = Arc::new(ToolRegistry::new());
        let server = McpServer::new(registry);
        let resources = server.handle_method("resources/list", None).await;
        assert!(resources["resources"].as_array().unwrap().is_empty());
        let prompts = server.handle_method("prompts/list", None).await;
        assert!(prompts["prompts"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn json_rpc_response_serialization() {
        let resp = JsonRpcResponse {
            jsonrpc: "2.0",
            id: json!(1),
            result: Some(json!({"ok": true})),
            error: None,
        };
        let serialized = resp.to_json().to_string();
        assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
        assert!(serialized.contains("\"id\":1"));
        assert!(serialized.contains("\"result\""));
        assert!(!serialized.contains("\"error\""));
    }

    #[tokio::test]
    async fn json_rpc_response_error_serialization() {
        let resp = JsonRpcResponse {
            jsonrpc: "2.0",
            id: Value::Null,
            result: None,
            error: Some(JsonRpcError {
                code: -32700,
                message: "parse error".to_string(),
            }),
        };
        let serialized = resp.to_json().to_string();
        assert!(serialized.contains("\"error\""));
        assert!(serialized.contains("-32700"));
    }
}
