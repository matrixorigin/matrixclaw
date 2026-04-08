use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use zstar_agent_core::event::AgentEvent;
use zstar_agent_core::message::RunMessageRole;
use zstar_agent_core::provider::{Provider, ProviderError, ProviderResponse};
use zstar_agent_core::{RunRequest, ToolCall, ToolChoice};

const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";

pub struct OpenAiProvider {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
    pub last_usage: Mutex<Option<crate::token_count::TokenUsage>>,
}

impl OpenAiProvider {
    pub fn for_openrouter(
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<Self, ProviderError> {
        Self::with_base_url(OPENROUTER_BASE_URL, api_key, model)
    }

    pub fn with_base_url(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<Self, ProviderError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(90))
            .build()
            .map_err(|error| ProviderError(format!("failed to construct HTTP client: {error}")))?;

        Ok(Self {
            client,
            base_url: base_url.into(),
            api_key: api_key.into(),
            model: model.into(),
            last_usage: Mutex::new(None),
        })
    }

    pub fn model_name(&self) -> &str {
        &self.model
    }

    fn completion_url(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }

    fn request_body(&self, request: &RunRequest, stream: bool) -> Value {
        let mut body = json!({
            "model": self.model,
            "messages": request_messages(request, &self.model),
            "stream": stream,
            "temperature": 0
        });

        if !request.tools.is_empty() {
            let tool_choice = match &request.tool_choice {
                ToolChoice::Auto => "auto",
                ToolChoice::None => "none",
            };
            body["tools"] = json!(request
                .tools
                .iter()
                .map(|t| t.to_openai_function())
                .collect::<Vec<_>>());
            body["tool_choice"] = json!(tool_choice);
        }

        body
    }

    async fn perform_completion(
        &self,
        request: &RunRequest,
    ) -> Result<ProviderResponse, ProviderError> {
        let response = self
            .client
            .post(self.completion_url())
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(CONTENT_TYPE, "application/json")
            .header("HTTP-Referer", "https://github.com/matrixorigin/zstar")
            .header("X-Title", "ZStar")
            .json(&self.request_body(request, false))
            .send()
            .await
            .map_err(|error| ProviderError(format!("provider request failed: {error}")))?;

        let status = response.status();
        let body: Value = response.json().await.map_err(|error| {
            ProviderError(format!("provider response was not valid JSON: {error}"))
        })?;

        if !status.is_success() {
            return Err(ProviderError(format!("provider returned {status}: {body}")));
        }

        if let Some(usage) = crate::token_count::TokenUsage::from_response(&body) {
            *self.last_usage.lock().unwrap() = Some(usage);
        }

        parse_provider_response(&body)
    }

    async fn perform_streaming_completion(
        &self,
        request: &RunRequest,
        on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<ProviderResponse, ProviderError> {
        let response = self
            .client
            .post(self.completion_url())
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(ACCEPT, "text/event-stream")
            .header(CONTENT_TYPE, "application/json")
            .header("HTTP-Referer", "https://github.com/matrixorigin/zstar")
            .header("X-Title", "ZStar")
            .json(&self.request_body(request, true))
            .send()
            .await
            .map_err(|error| ProviderError(format!("provider request failed: {error}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.map_err(|error| {
                ProviderError(format!("provider error body was unreadable: {error}"))
            })?;
            return Err(ProviderError(format!("provider returned {status}: {body}")));
        }

        let mut final_content = String::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut event_data = Vec::new();
        let mut buffer = String::new();

        let byte_stream = response.bytes_stream();
        use futures_util::StreamExt;
        let mut stream = byte_stream.boxed();

        let mut last_payload: Option<String> = None;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result
                .map_err(|error| ProviderError(format!("provider stream read error: {error}")))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim_end_matches('\r').to_string();
                buffer = buffer[pos + 1..].to_string();

                if line.is_empty() {
                    if let Some(payload) = drain_sse_event(&mut event_data) {
                        if payload == "[DONE]" {
                            continue;
                        }
                        last_payload = Some(payload.clone());
                        handle_stream_chunk_into(
                            payload,
                            &mut final_content,
                            &mut tool_calls,
                            on_event,
                        )?;
                    }
                    continue;
                }

                if let Some(rest) = line.strip_prefix("data:") {
                    let value = rest.strip_prefix(' ').unwrap_or(rest);
                    event_data.push(value.to_string());
                }
            }
        }

        if let Some(payload) = drain_sse_event(&mut event_data) {
            if payload != "[DONE]" {
                last_payload = Some(payload.clone());
                handle_stream_chunk_into(payload, &mut final_content, &mut tool_calls, on_event)?;
            }
        }

        if let Some(ref payload) = last_payload {
            if let Ok(value) = serde_json::from_str::<Value>(payload) {
                if let Some(usage) = crate::token_count::TokenUsage::from_response(&value) {
                    *self.last_usage.lock().unwrap() = Some(usage);
                }
            }
        }

        if !tool_calls.is_empty() {
            let parsed: Vec<ToolCall> = tool_calls
                .into_iter()
                .map(|mut tc| {
                    if let Some(s) = tc.arguments.as_str() {
                        tc.arguments = serde_json::from_str(s).unwrap_or(serde_json::Value::Null);
                    }
                    tc
                })
                .collect();
            return Ok(ProviderResponse::tool_calls(parsed));
        }

        if final_content.is_empty() {
            return Err(ProviderError(
                "provider response did not include assistant content".to_string(),
            ));
        }

        Ok(ProviderResponse::text(final_content))
    }
}

#[async_trait]
impl Provider for OpenAiProvider {
    async fn complete(&mut self, request: &RunRequest) -> Result<ProviderResponse, ProviderError> {
        self.perform_completion(request).await
    }

    async fn stream(
        &mut self,
        request: &RunRequest,
        on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<ProviderResponse, ProviderError> {
        self.perform_streaming_completion(request, on_event).await
    }
}

fn should_cache(model: &str) -> bool {
    let lower = model.to_lowercase();
    lower.contains("claude") || lower.contains("anthropic") || lower.contains("gemini")
}

fn request_messages(request: &RunRequest, model: &str) -> Vec<Value> {
    let messages = if request.context_messages.is_empty() {
        vec![zstar_agent_core::RunMessage::user(request.prompt.clone())]
    } else {
        request.context_messages.clone()
    };

    let mut msgs: Vec<Value> = messages.into_iter().map(message_to_openai).collect();

    if should_cache(model) {
        let user_indices: Vec<usize> = msgs
            .iter()
            .enumerate()
            .filter(|(_, m)| m.get("role").and_then(Value::as_str) == Some("user"))
            .map(|(i, _)| i)
            .collect();

        let last_3_users: Vec<usize> = user_indices.into_iter().rev().take(3).collect();

        for (i, msg) in msgs.iter_mut().enumerate() {
            let is_system = msg.get("role").and_then(Value::as_str) == Some("system") && i == 0;
            let is_cached_user = last_3_users.contains(&i);

            if is_system || is_cached_user {
                msg["cache_control"] = json!({"type": "ephemeral"});
            }
        }
    }

    msgs
}

fn message_to_openai(message: zstar_agent_core::RunMessage) -> Value {
    let role = match message.role {
        RunMessageRole::User => "user",
        RunMessageRole::System => "system",
        RunMessageRole::Assistant => "assistant",
        RunMessageRole::Tool => "tool",
    };

    let mut msg = json!({
        "role": role,
        "content": message.content
    });

    if let Some(tool_call_id) = message.tool_call_id {
        msg["tool_call_id"] = json!(tool_call_id);
    }

    if let Some(calls) = message.tool_calls {
        let openai_calls: Vec<Value> = calls
            .iter()
            .map(|call| {
                json!({
                    "id": call.id,
                    "type": "function",
                    "function": {
                        "name": call.name,
                        "arguments": call.arguments.to_string()
                    }
                })
            })
            .collect();
        msg["tool_calls"] = json!(openai_calls);
    }

    msg
}

fn parse_provider_response(body: &Value) -> Result<ProviderResponse, ProviderError> {
    let choice = body
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("message"))
        .ok_or_else(|| ProviderError(format!("missing message in response: {body}")))?;

    let tool_calls = parse_tool_calls_from_response(choice);
    if !tool_calls.is_empty() {
        return Ok(ProviderResponse::tool_calls(tool_calls));
    }

    let content = extract_message_content_from_choice(choice);
    match content {
        Some(text) if !text.trim().is_empty() => {
            Ok(ProviderResponse::text(text.trim().to_string()))
        }
        _ => Err(ProviderError(format!(
            "provider response did not include assistant content: {body}"
        ))),
    }
}

fn parse_tool_calls_from_response(message: &Value) -> Vec<ToolCall> {
    let calls = match message.get("tool_calls").and_then(Value::as_array) {
        Some(calls) => calls,
        None => return Vec::new(),
    };

    calls
        .iter()
        .filter_map(|call| {
            let id = call.get("id")?.as_str()?.to_string();
            let func = call.get("function")?;
            let name = func.get("name")?.as_str()?.to_string();
            let args_str = func.get("arguments")?.as_str().unwrap_or("{}");
            let arguments = serde_json::from_str(args_str).ok()?;
            Some(ToolCall::new(id, name, arguments))
        })
        .collect()
}

fn drain_sse_event(event_data: &mut Vec<String>) -> Option<String> {
    if event_data.is_empty() {
        return None;
    }
    let payload = event_data.join("\n");
    event_data.clear();
    Some(payload)
}

fn handle_stream_chunk_into(
    payload: String,
    final_content: &mut String,
    tool_calls: &mut Vec<ToolCall>,
    on_event: &mut dyn FnMut(AgentEvent),
) -> Result<(), ProviderError> {
    let chunk: ChatCompletionChunk = serde_json::from_str(&payload).map_err(|error| {
        ProviderError(format!(
            "provider stream chunk was not valid JSON: {error}; payload={payload}"
        ))
    })?;

    if let Some(choice) = chunk.choices.first() {
        if let Some(content) = &choice.delta.content {
            let text = extract_text_from_value(content);
            if !text.is_empty() {
                on_event(AgentEvent::MessageDelta(text.clone()));
                final_content.push_str(&text);
            }
        }

        if let Some(calls) = &choice.delta.tool_calls {
            for stream_call in calls {
                let idx = stream_call.index as usize;
                while tool_calls.len() <= idx {
                    tool_calls.push(ToolCall::new(
                        String::new(),
                        String::new(),
                        serde_json::Value::Null,
                    ));
                }
                if let Some(id) = &stream_call.id {
                    tool_calls[idx].id = id.clone();
                }
                if let Some(func) = &stream_call.function {
                    if let Some(name) = &func.name {
                        tool_calls[idx].name = name.clone();
                    }
                    if let Some(args) = &func.arguments {
                        if tool_calls[idx].arguments.is_null() {
                            tool_calls[idx].arguments = serde_json::Value::String(String::new());
                        }
                        if let Some(s) = tool_calls[idx].arguments.as_str() {
                            let mut combined = s.to_string();
                            combined.push_str(args);
                            tool_calls[idx].arguments = serde_json::Value::String(combined);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn extract_text_from_value(content: &Value) -> String {
    if let Some(text) = content.as_str() {
        return text.to_string();
    }

    if let Some(parts) = content.as_array() {
        let mut text = String::new();
        for part in parts {
            if let Some(value) = part.get("text").and_then(Value::as_str) {
                text.push_str(value);
                continue;
            }
            if let Some(value) = part.get("content").and_then(Value::as_str) {
                text.push_str(value);
            }
        }
        return text;
    }

    String::new()
}

fn extract_message_content_from_choice(message: &Value) -> Option<String> {
    let content = message.get("content")?;

    if let Some(text) = content.as_str() {
        return Some(text.to_string());
    }

    let parts = content.as_array()?;
    let mut text = String::new();
    for part in parts {
        if let Some(value) = part.get("text").and_then(Value::as_str) {
            text.push_str(value);
            continue;
        }
        if let Some(value) = part.get("content").and_then(Value::as_str) {
            text.push_str(value);
        }
    }

    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    #[serde(default)]
    content: Option<Value>,
    #[serde(default)]
    tool_calls: Option<Vec<StreamToolCall>>,
}

#[derive(Debug, Deserialize)]
struct StreamToolCall {
    #[serde(default)]
    index: u32,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    function: Option<StreamFunction>,
}

#[derive(Debug, Deserialize)]
struct StreamFunction {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_plain_string_content() {
        let body = json!({
            "choices": [
                {
                    "message": {
                        "content": "ZStar"
                    }
                }
            ]
        });

        let response = parse_provider_response(&body).unwrap();
        assert_eq!(response.content.as_deref(), Some("ZStar"));
    }

    #[test]
    fn extracts_text_from_content_parts() {
        let body = json!({
            "choices": [
                {
                    "message": {
                        "content": [
                            { "type": "text", "text": "Z" },
                            { "type": "text", "text": "Star" }
                        ]
                    }
                }
            ]
        });

        let response = parse_provider_response(&body).unwrap();
        assert_eq!(response.content.as_deref(), Some("ZStar"));
    }

    #[test]
    fn extracts_tool_calls() {
        let body = json!({
            "choices": [
                {
                    "message": {
                        "content": null,
                        "tool_calls": [{
                            "id": "call_abc123",
                            "type": "function",
                            "function": {
                                "name": "terminal",
                                "arguments": "{\"command\": \"ls\"}"
                            }
                        }]
                    }
                }
            ]
        });

        let response = parse_provider_response(&body).unwrap();
        assert!(response.is_tool_call());
        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].name, "terminal");
    }

    #[test]
    fn cache_control_added_for_claude_models() {
        use zstar_agent_core::RunMessage;

        let request = RunRequest {
            prompt: "current prompt".into(),
            context_messages: vec![
                RunMessage::system("system instructions"),
                RunMessage::user("first user"),
                RunMessage::assistant("response 1"),
                RunMessage::user("second user"),
                RunMessage::assistant("response 2"),
                RunMessage::user("third user"),
                RunMessage::assistant("response 3"),
                RunMessage::user("fourth user"),
            ],
            tools: vec![],
            tool_choice: ToolChoice::Auto,
            max_iterations: 10,
        };

        let messages = request_messages(&request, "anthropic/claude-3.5-sonnet");

        let system_msg = &messages[0];
        assert_eq!(system_msg["role"], "system");
        assert_eq!(system_msg["cache_control"], json!({"type": "ephemeral"}));

        let user_indices: Vec<usize> = messages
            .iter()
            .enumerate()
            .filter(|(_, m)| m["role"] == "user")
            .map(|(i, _)| i)
            .collect();

        assert_eq!(user_indices.len(), 4);

        assert_eq!(
            messages[user_indices[0]]["cache_control"],
            Value::Null,
            "first user should NOT have cache_control"
        );
        for &idx in &user_indices[1..] {
            assert_eq!(
                messages[idx]["cache_control"],
                json!({"type": "ephemeral"}),
                "last 3 user messages should have cache_control at index {idx}"
            );
        }
    }

    #[test]
    fn no_cache_control_for_non_claude() {
        use zstar_agent_core::RunMessage;

        let request = RunRequest {
            prompt: "current prompt".into(),
            context_messages: vec![
                RunMessage::system("system instructions"),
                RunMessage::user("hello"),
            ],
            tools: vec![],
            tool_choice: ToolChoice::Auto,
            max_iterations: 10,
        };

        let messages = request_messages(&request, "openai/gpt-4o");

        for msg in &messages {
            assert!(
                msg.get("cache_control").is_none(),
                "non-claude model should not have cache_control"
            );
        }
    }

    #[test]
    fn cache_control_added_for_gemini_models() {
        use zstar_agent_core::RunMessage;

        let request = RunRequest {
            prompt: "current prompt".into(),
            context_messages: vec![
                RunMessage::system("system instructions"),
                RunMessage::user("hello"),
            ],
            tools: vec![],
            tool_choice: ToolChoice::Auto,
            max_iterations: 10,
        };

        let messages = request_messages(&request, "google/gemini-2.5-flash");

        assert_eq!(messages[0]["cache_control"], json!({"type": "ephemeral"}));
        assert_eq!(messages[1]["cache_control"], json!({"type": "ephemeral"}));
    }

    #[test]
    fn cache_control_with_fewer_than_three_user_messages() {
        use zstar_agent_core::RunMessage;

        let request = RunRequest {
            prompt: "current prompt".into(),
            context_messages: vec![
                RunMessage::system("system instructions"),
                RunMessage::user("only user"),
            ],
            tools: vec![],
            tool_choice: ToolChoice::Auto,
            max_iterations: 10,
        };

        let messages = request_messages(&request, "anthropic/claude-3.5-sonnet");

        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[0]["cache_control"], json!({"type": "ephemeral"}));
        assert_eq!(messages[1]["role"], "user");
        assert_eq!(messages[1]["cache_control"], json!({"type": "ephemeral"}));
    }

    #[test]
    fn cache_control_default_prompt_no_context() {
        let request = RunRequest::new("hello");

        let messages = request_messages(&request, "anthropic/claude-3.5-sonnet");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[0]["cache_control"], json!({"type": "ephemeral"}));
    }
}
