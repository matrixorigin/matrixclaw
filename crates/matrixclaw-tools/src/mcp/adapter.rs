use async_trait::async_trait;
use std::sync::Arc;

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

use super::client::McpClient;
use super::types::McpTool;

pub struct McpToolAdapter {
    descriptor: ToolDescriptor,
    client: Arc<McpClient>,
    mcp_name: String,
}

impl McpToolAdapter {
    pub fn new(tool: &McpTool, client: Arc<McpClient>) -> Self {
        let descriptor = convert_descriptor(tool);
        Self {
            descriptor,
            client,
            mcp_name: tool.name.clone(),
        }
    }
}

impl std::fmt::Debug for McpToolAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpToolAdapter")
            .field("mcp_name", &self.mcp_name)
            .finish()
    }
}

#[async_trait]
impl ToolExecutor for McpToolAdapter {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let args = call.arguments.clone();
        match self.client.call_tool(&self.mcp_name, args).await {
            Ok(result) => {
                let output = result.text_output();
                if result.is_error() {
                    ToolResult::error(&call, output)
                } else {
                    ToolResult::success(&call, output)
                }
            }
            Err(e) => ToolResult::error(&call, format!("MCP tool call failed: {e}")),
        }
    }
}

fn convert_descriptor(tool: &McpTool) -> ToolDescriptor {
    let parameters = tool
        .input_schema
        .as_ref()
        .and_then(|schema| schema.get("properties"))
        .and_then(|props| props.as_object())
        .map(|props| {
            let required_keys = tool
                .input_schema
                .as_ref()
                .and_then(|s| s.get("required"))
                .and_then(|r| r.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                .unwrap_or_default();

            props
                .iter()
                .map(|(name, schema)| {
                    let param_type = schema
                        .get("type")
                        .and_then(|t| t.as_str())
                        .map(parse_json_type)
                        .unwrap_or(ParameterType::String);

                    let description = schema
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string();

                    let is_required = required_keys.iter().any(|k| *k == name);

                    ToolParameter {
                        name: name.clone(),
                        param_type,
                        description,
                        required: is_required,
                        enum_values: None,
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    ToolDescriptor {
        name: tool.name.clone(),
        description: tool
            .description
            .clone()
            .unwrap_or_else(|| "MCP tool".to_string()),
        parameters,
    }
}

fn parse_json_type(t: &str) -> ParameterType {
    match t {
        "number" => ParameterType::Number,
        "integer" => ParameterType::Integer,
        "boolean" => ParameterType::Boolean,
        "array" => ParameterType::Array,
        "object" => ParameterType::Object,
        _ => ParameterType::String,
    }
}
