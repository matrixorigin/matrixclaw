use async_trait::async_trait;

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct WebFetchTool {
    descriptor: ToolDescriptor,
    client: reqwest::Client,
}

impl Default for WebFetchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl WebFetchTool {
    pub fn new() -> Self {
        Self {
            descriptor: ToolDescriptor::new("web_fetch", "Fetch content from a URL")
                .with_parameters(vec![
                    ToolParameter::required("url", ParameterType::String, "URL to fetch"),
                    ToolParameter::optional(
                        "method",
                        ParameterType::String,
                        "HTTP method (default GET)",
                    ),
                    ToolParameter::optional(
                        "headers",
                        ParameterType::Object,
                        "HTTP headers to include",
                    ),
                ]),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }
}

#[async_trait]
impl ToolExecutor for WebFetchTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let url = match call.arguments.get("url").and_then(|v| v.as_str()) {
            Some(u) => u,
            None => return ToolResult::error(&call, "missing required parameter: url"),
        };

        let method = call
            .arguments
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET");

        let mut request = match method {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "PUT" => self.client.put(url),
            "DELETE" => self.client.delete(url),
            "HEAD" => self.client.head(url),
            "PATCH" => self.client.patch(url),
            _ => return ToolResult::error(&call, format!("unsupported HTTP method: {method}")),
        };

        if let Some(headers) = call.arguments.get("headers").and_then(|v| v.as_object()) {
            for (key, value) in headers {
                if let Some(val_str) = value.as_str() {
                    request = request.header(key, val_str);
                }
            }
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status();
                match response.text().await {
                    Ok(body) => {
                        let truncated = if body.len() > 50000 {
                            format!(
                                "{}...\n(truncated, {} chars total)",
                                &body[..50000],
                                body.len()
                            )
                        } else {
                            body
                        };
                        ToolResult::success(&call, format!("HTTP {status}:\n{truncated}"))
                    }
                    Err(e) => {
                        ToolResult::error(&call, format!("failed to read response body: {e}"))
                    }
                }
            }
            Err(e) => ToolResult::error(&call, format!("request failed: {e}")),
        }
    }
}

pub struct WebSearchTool {
    descriptor: ToolDescriptor,
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSearchTool {
    pub fn new() -> Self {
        Self {
            descriptor: ToolDescriptor::new("web_search", "Search the web for information")
                .with_parameters(vec![ToolParameter::required(
                    "query",
                    ParameterType::String,
                    "Search query",
                )]),
        }
    }
}

#[async_trait]
impl ToolExecutor for WebSearchTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        ToolResult::error(
            &call,
            "web_search requires a search provider API key (not yet configured)",
        )
    }
}
