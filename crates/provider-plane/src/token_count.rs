use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
pub struct TokenUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
}

impl TokenUsage {
    pub fn from_response(body: &Value) -> Option<Self> {
        let usage = body.get("usage")?;
        Some(Self {
            prompt_tokens: usage.get("prompt_tokens")?.as_u64()?,
            completion_tokens: usage.get("completion_tokens")?.as_u64()?,
        })
    }

    pub fn total(&self) -> u64 {
        self.prompt_tokens + self.completion_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_usage_from_response() {
        let body = serde_json::json!({
            "usage": {
                "prompt_tokens": 150,
                "completion_tokens": 80
            }
        });
        let usage = TokenUsage::from_response(&body).unwrap();
        assert_eq!(usage.prompt_tokens, 150);
        assert_eq!(usage.completion_tokens, 80);
    }

    #[test]
    fn total_sums_tokens() {
        let usage = TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 50,
        };
        assert_eq!(usage.total(), 150);
    }
}
