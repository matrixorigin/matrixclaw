use async_trait::async_trait;

use crate::config::{SandboxConfig, SandboxKind};
use crate::error::SandboxError;
use crate::runtime::{CodeRequest, CommandRequest, SandboxResult, SandboxRuntime};

pub struct DaytonaBackend {
    api_key: String,
    server_url: String,
    timeout_secs: u64,
}

impl DaytonaBackend {
    pub fn new(config: &SandboxConfig) -> Result<Self, SandboxError> {
        let api_key = config
            .daytona_api_key
            .clone()
            .or_else(|| std::env::var("DAYTONA_API_KEY").ok())
            .ok_or_else(|| {
                SandboxError::Config(
                    "Daytona API key not provided and DAYTONA_API_KEY not set".into(),
                )
            })?;
        let server_url = config
            .daytona_server_url
            .clone()
            .or_else(|| std::env::var("DAYTONA_SERVER_URL").ok())
            .unwrap_or_else(|| "https://api.daytona.io".into());
        Ok(Self {
            api_key,
            server_url,
            timeout_secs: config.default_timeout_secs,
        })
    }

    fn client(&self) -> reqwest::Client {
        reqwest::Client::new()
    }
}

#[async_trait]
impl SandboxRuntime for DaytonaBackend {
    fn kind(&self) -> SandboxKind {
        SandboxKind::Daytona
    }

    fn is_available(&self) -> bool {
        std::env::var("DAYTONA_API_KEY").is_ok()
    }

    async fn execute_code(&self, request: CodeRequest) -> Result<SandboxResult, SandboxError> {
        let timeout_secs = request.timeout_secs.unwrap_or(self.timeout_secs);
        let client = self.client();
        let url = format!("{}/sandbox/execute", self.server_url.trim_end_matches('/'));

        let result = tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), async {
            let body = serde_json::json!({
                "code": request.code,
                "language": request.language,
            });

            let resp = client
                .post(&url)
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

            if !status.is_success() {
                return Ok::<_, SandboxError>(SandboxResult::failure(
                    status.as_u16() as i32,
                    text,
                    SandboxKind::Daytona,
                ));
            }

            Ok(SandboxResult::success(text, SandboxKind::Daytona))
        })
        .await
        .map_err(|_| SandboxError::Timeout { secs: timeout_secs })??;

        Ok(result)
    }

    async fn execute_command(
        &self,
        request: CommandRequest,
    ) -> Result<SandboxResult, SandboxError> {
        let timeout_secs = request.timeout_secs.unwrap_or(self.timeout_secs);
        let client = self.client();
        let url = format!("{}/sandbox/command", self.server_url.trim_end_matches('/'));
        let full_cmd = if request.args.is_empty() {
            request.command
        } else {
            format!("{} {}", request.command, request.args.join(" "))
        };

        let result = tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), async {
            let body = serde_json::json!({
                "command": full_cmd,
            });

            let resp = client
                .post(&url)
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

            if !status.is_success() {
                return Ok::<_, SandboxError>(SandboxResult::failure(
                    status.as_u16() as i32,
                    text,
                    SandboxKind::Daytona,
                ));
            }

            Ok(SandboxResult::success(text, SandboxKind::Daytona))
        })
        .await
        .map_err(|_| SandboxError::Timeout { secs: timeout_secs })??;

        Ok(result)
    }

    fn supported_languages(&self) -> Vec<String> {
        vec![
            "python".into(),
            "javascript".into(),
            "typescript".into(),
            "bash".into(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_requires_api_key() {
        let config = SandboxConfig {
            kind: SandboxKind::Daytona,
            daytona_api_key: None,
            ..SandboxConfig::default()
        };
        std::env::remove_var("DAYTONA_API_KEY");
        assert!(DaytonaBackend::new(&config).is_err());
    }

    #[test]
    fn new_accepts_config_key() {
        let config = SandboxConfig {
            kind: SandboxKind::Daytona,
            daytona_api_key: Some("test-key".into()),
            ..SandboxConfig::default()
        };
        assert!(DaytonaBackend::new(&config).is_ok());
    }

    #[test]
    fn supported_languages() {
        let backend = DaytonaBackend {
            api_key: "test".into(),
            server_url: "http://localhost".into(),
            timeout_secs: 30,
        };
        assert!(backend
            .supported_languages()
            .contains(&"python".to_string()));
    }
}
