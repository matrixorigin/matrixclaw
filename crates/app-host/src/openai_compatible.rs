use std::time::Duration;
use std::io::{BufRead, BufReader};

use matrixclaw_agent_core::event::AgentEvent;
use matrixclaw_agent_core::provider::{Provider, ProviderError};
use matrixclaw_agent_core::{RunMessage, RunMessageRole, RunRequest};
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use serde_json::{json, Value};

const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";

pub struct OpenAiCompatibleProvider {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl OpenAiCompatibleProvider {
    pub fn for_openrouter(api_key: impl Into<String>, model: impl Into<String>) -> Result<Self, ProviderError> {
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
        })
    }

    fn completion_url(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }

    fn request_body(&self, request: &RunRequest) -> Value {
        json!({
            "model": self.model,
            "messages": request_messages(request),
            "stream": false,
            "temperature": 0
        })
    }

    fn perform_completion(&self, request: &RunRequest) -> Result<String, ProviderError> {
        let response = self
            .client
            .post(self.completion_url())
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(CONTENT_TYPE, "application/json")
            .header("HTTP-Referer", "https://github.com/matrixorigin/matrixclaw")
            .header("X-Title", "MatrixClaw")
            .json(&self.request_body(request))
            .send()
            .map_err(|error| ProviderError(format!("provider request failed: {error}")))?;

        let status = response.status();
        let body: Value = response
            .json()
            .map_err(|error| ProviderError(format!("provider response was not valid JSON: {error}")))?;

        if !status.is_success() {
            return Err(ProviderError(format!(
                "provider returned {}: {}",
                status,
                body
            )));
        }

        extract_message_content(&body)
            .map(|content| content.trim().to_string())
            .filter(|content| !content.is_empty())
            .ok_or_else(|| ProviderError(format!("provider response did not include assistant content: {body}")))
    }

    fn perform_streaming_completion(
        &self,
        request: &RunRequest,
        on_event: &mut dyn FnMut(AgentEvent),
    ) -> Result<String, ProviderError> {
        let response = self
            .client
            .post(self.completion_url())
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(ACCEPT, "text/event-stream")
            .header(CONTENT_TYPE, "application/json")
            .header("HTTP-Referer", "https://github.com/matrixorigin/matrixclaw")
            .header("X-Title", "MatrixClaw")
            .json(&request_body_with_stream(&self.model, request, true))
            .send()
            .map_err(|error| ProviderError(format!("provider request failed: {error}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .map_err(|error| ProviderError(format!("provider error body was unreadable: {error}")))?;
            return Err(ProviderError(format!("provider returned {}: {}", status, body)));
        }

        let mut reader = BufReader::new(response);
        let mut final_message = String::new();
        let mut event_data = Vec::new();

        on_event(AgentEvent::RunStarted);
        on_event(AgentEvent::MessageStarted);

        loop {
            let mut line = String::new();
            let bytes_read = reader
                .read_line(&mut line)
                .map_err(|error| ProviderError(format!("provider stream could not be read: {error}")))?;

            if bytes_read == 0 {
                break;
            }

            let line = line.trim_end_matches(['\r', '\n']);
            if line.is_empty() {
                if let Some(payload) = drain_sse_event(&mut event_data) {
                    if payload == "[DONE]" {
                        break;
                    }
                    final_message.push_str(&handle_stream_chunk(payload, on_event)?);
                }
                continue;
            }

            if let Some(rest) = line.strip_prefix("data:") {
                let value = rest.strip_prefix(' ').unwrap_or(rest);
                event_data.push(value.to_string());
            }
        }

        if let Some(payload) = drain_sse_event(&mut event_data) {
            if payload != "[DONE]" {
                final_message.push_str(&handle_stream_chunk(payload, on_event)?);
            }
        }

        if final_message.is_empty() {
            return Err(ProviderError("provider response did not include assistant content".to_string()));
        }

        on_event(AgentEvent::MessageCompleted(final_message.clone()));
        Ok(final_message)
    }
}

impl Provider for OpenAiCompatibleProvider {
    fn complete(&mut self, request: &RunRequest) -> Result<String, ProviderError> {
        self.perform_completion(request)
    }

    fn stream(
        &mut self,
        request: &RunRequest,
        on_event: &mut dyn FnMut(AgentEvent),
    ) -> Result<String, ProviderError> {
        self.perform_streaming_completion(request, on_event)
    }
}

fn request_body_with_stream(model: &str, request: &RunRequest, stream: bool) -> Value {
    json!({
        "model": model,
        "messages": request_messages(request),
        "stream": stream,
        "temperature": 0
    })
}

fn request_messages(request: &RunRequest) -> Vec<Value> {
    let messages = if request.context_messages.is_empty() {
        vec![RunMessage::user(request.prompt.clone())]
    } else {
        request.context_messages.clone()
    };

    messages
        .into_iter()
        .map(|message| {
            json!({
                "role": request_role(&message.role),
                "content": message.content
            })
        })
        .collect()
}

fn request_role(role: &RunMessageRole) -> &'static str {
    match role {
        RunMessageRole::User => "user",
        RunMessageRole::System => "system",
        RunMessageRole::Assistant => "assistant",
        RunMessageRole::Tool => "tool",
    }
}

fn drain_sse_event(event_data: &mut Vec<String>) -> Option<String> {
    if event_data.is_empty() {
        return None;
    }

    let payload = event_data.join("\n");
    event_data.clear();
    Some(payload)
}

fn handle_stream_chunk(
    payload: String,
    on_event: &mut dyn FnMut(AgentEvent),
) -> Result<String, ProviderError> {
    let chunk: ChatCompletionChunk = serde_json::from_str(&payload)
        .map_err(|error| ProviderError(format!("provider stream chunk was not valid JSON: {error}; payload={payload}")))?;

    let mut text = String::new();
    if let Some(choice) = chunk.choices.first() {
        if let Some(content) = extract_delta_content(&choice.delta.content) {
            if !content.is_empty() {
                on_event(AgentEvent::MessageDelta(content.clone()));
                text.push_str(&content);
            }
        }
    }

    Ok(text)
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChunk {
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChoice {
    delta: ChatCompletionDelta,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionDelta {
    #[serde(default)]
    content: Option<Value>,
}

fn extract_delta_content(content: &Option<Value>) -> Option<String> {
    let content = content.as_ref()?;

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

fn extract_message_content(body: &Value) -> Option<String> {
    let content = body.get("choices")?.as_array()?.first()?.get("message")?.get("content")?;

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

#[cfg(test)]
mod tests {
    use super::extract_message_content;
    use serde_json::json;

    #[test]
    fn extracts_plain_string_content() {
        let body = json!({
            "choices": [
                {
                    "message": {
                        "content": "MatrixClaw"
                    }
                }
            ]
        });

        assert_eq!(extract_message_content(&body).as_deref(), Some("MatrixClaw"));
    }

    #[test]
    fn extracts_text_from_content_parts() {
        let body = json!({
            "choices": [
                {
                    "message": {
                        "content": [
                            { "type": "text", "text": "Matrix" },
                            { "type": "text", "text": "Claw" }
                        ]
                    }
                }
            ]
        });

        assert_eq!(extract_message_content(&body).as_deref(), Some("MatrixClaw"));
    }
}
