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
            descriptor: ToolDescriptor::new(
                "web_search",
                "Search the web for information. Returns up to 5 results with titles, URLs, and snippets.",
            )
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
        let query = match call.arguments.get("query").and_then(|v| v.as_str()) {
            Some(q) => q,
            None => return ToolResult::error(&call, "missing required parameter: query"),
        };

        let client = reqwest::Client::new();
        let url = format!(
            "https://searx.be/search?q={}&format=json&categories=general",
            urlencoding::encode(query),
        );

        let response = match client
            .get(&url)
            .header("User-Agent", "ZStar/0.1")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => return ToolResult::error(&call, format!("search request failed: {e}")),
        };

        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(e) => {
                return ToolResult::error(&call, format!("search response parse failed: {e}"))
            }
        };

        let results: Vec<String> = body["results"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .take(5)
            .map(|r| {
                let title = r["title"].as_str().unwrap_or("");
                let url = r["url"].as_str().unwrap_or("");
                let snippet = r["content"].as_str().unwrap_or("");
                format!("{title}\n  {url}\n  {snippet}")
            })
            .collect();

        if results.is_empty() {
            ToolResult::success(&call, "no results found")
        } else {
            ToolResult::success(&call, results.join("\n\n"))
        }
    }
}

#[cfg(test)]
mod search_tests {
    use super::*;

    #[tokio::test]
    async fn descriptor_correct() {
        let tool = WebSearchTool::new();
        assert_eq!(tool.descriptor().name, "web_search");
    }

    #[tokio::test]
    #[ignore]
    async fn search_returns_results() {
        let tool = WebSearchTool::new();
        let call = ToolCall::new(
            "1".into(),
            "web_search".into(),
            serde_json::json!({"query": "rust programming language"}),
        );
        let result = tool.execute(call).await;
        assert!(!result.is_error);
        assert!(result.output.contains("rust"));
    }
}
