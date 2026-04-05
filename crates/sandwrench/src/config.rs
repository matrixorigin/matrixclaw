use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub kind: SandboxKind,
    pub default_timeout_secs: u64,
    pub memory_limit: String,
    pub cpu_limit: f64,
    pub network_enabled: bool,
    #[serde(default)]
    pub docker_image: Option<String>,
    #[serde(default)]
    pub e2b_api_key: Option<String>,
    #[serde(default)]
    pub daytona_api_key: Option<String>,
    #[serde(default)]
    pub daytona_server_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SandboxKind {
    Docker,
    E2b,
    Daytona,
    Wasm,
    Local,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            kind: SandboxKind::Docker,
            default_timeout_secs: 30,
            memory_limit: "512m".into(),
            cpu_limit: 1.0,
            network_enabled: false,
            docker_image: Some("ubuntu:22.04".into()),
            e2b_api_key: None,
            daytona_api_key: None,
            daytona_server_url: None,
        }
    }
}
