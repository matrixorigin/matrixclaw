use async_trait::async_trait;

use crate::config::{SandboxConfig, SandboxKind};
use crate::error::SandboxError;
use crate::runtime::{CodeRequest, CommandRequest, SandboxResult, SandboxRuntime};

pub struct E2bBackend {
    api_key: String,
    timeout_secs: u64,
}

impl E2bBackend {
    pub fn new(config: &SandboxConfig) -> Result<Self, SandboxError> {
        let api_key = config
            .e2b_api_key
            .clone()
            .or_else(|| std::env::var("E2B_API_KEY").ok())
            .ok_or_else(|| {
                SandboxError::Config("E2B API key not provided and E2B_API_KEY not set".into())
            })?;
        Ok(Self {
            api_key,
            timeout_secs: config.default_timeout_secs,
        })
    }

    fn client(&self) -> reqwest::Client {
        reqwest::Client::new()
    }
}

#[async_trait]
impl SandboxRuntime for E2bBackend {
    fn kind(&self) -> SandboxKind {
        SandboxKind::E2b
    }

    fn is_available(&self) -> bool {
        std::env::var("E2B_API_KEY").is_ok()
    }

    async fn execute_code(&self, request: CodeRequest) -> Result<SandboxResult, SandboxError> {
        let timeout_secs = request.timeout_secs.unwrap_or(self.timeout_secs);
        let client = self.client();

        let body = serde_json::json!({
            "code": request.code,
            "language": request.language,
        });

        let fut = async {
            let resp = client
                .post("https://api.e2b.app/v1/sandboxes")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| SandboxError::HttpError(e.to_string()))?;

            let status = resp.status();
            let text = resp
                .text()
                .await
                .map_err(|e| SandboxError::HttpError(e.to_string()))?;

            Ok::<SandboxResult, SandboxError>(if !status.is_success() {
                SandboxResult::failure(status.as_u16() as i32, text, SandboxKind::E2b)
            } else {
                SandboxResult::success(text, SandboxKind::E2b)
            })
        };

        tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), fut)
            .await
            .map_err(|_| SandboxError::Timeout { secs: timeout_secs })?
    }

    async fn execute_command(
        &self,
        _request: CommandRequest,
    ) -> Result<SandboxResult, SandboxError> {
        Err(SandboxError::BackendUnavailable(
            "E2B does not support raw command execution".into(),
        ))
    }

    fn supported_languages(&self) -> Vec<String> {
        vec!["python".into(), "javascript".into(), "bash".into()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_requires_api_key() {
        let config = SandboxConfig {
            kind: SandboxKind::E2b,
            e2b_api_key: None,
            ..SandboxConfig::default()
        };
        std::env::remove_var("E2B_API_KEY");
        assert!(E2bBackend::new(&config).is_err());
    }

    #[test]
    fn new_accepts_config_key() {
        let config = SandboxConfig {
            kind: SandboxKind::E2b,
            e2b_api_key: Some("test-key".into()),
            ..SandboxConfig::default()
        };
        assert!(E2bBackend::new(&config).is_ok());
    }

    #[test]
    fn supported_languages() {
        let backend = E2bBackend {
            api_key: "test".into(),
            timeout_secs: 30,
        };
        assert!(backend
            .supported_languages()
            .contains(&"python".to_string()));
    }
}
