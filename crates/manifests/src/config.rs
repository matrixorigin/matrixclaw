use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub provider: ProviderSettings,
    pub workspace: WorkspaceSettings,
    pub auth: AuthSettings,
    pub managed_assets: ManagedAssetsSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    Local,
    Sandboxed,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionBackendKind {
    LocalCommand,
    Sandbox,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionBackendSelection {
    pub kind: ExecutionBackendKind,
    pub label: String,
    pub requires_docker: bool,
}

impl ExecutionBackendSelection {
    pub fn local_command() -> Self {
        Self {
            kind: ExecutionBackendKind::LocalCommand,
            label: "local-command".to_string(),
            requires_docker: false,
        }
    }

    pub fn sandbox() -> Self {
        Self {
            kind: ExecutionBackendKind::Sandbox,
            label: "sandbox".to_string(),
            requires_docker: false,
        }
    }

    pub fn disabled() -> Self {
        Self {
            kind: ExecutionBackendKind::Disabled,
            label: "disabled".to_string(),
            requires_docker: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionSettings {
    pub mode: ExecutionMode,
    pub backend: ExecutionBackendSelection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxPolicySettings {
    pub enabled: bool,
    pub backend: ExecutionBackendSelection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupWizardSubmission {
    pub provider: ProviderSettings,
    pub workspace: WorkspaceSettings,
    pub auth: AuthSettings,
    pub execution: ExecutionSettings,
}

impl SetupWizardSubmission {
    pub fn new(
        provider: ProviderSettings,
        workspace: WorkspaceSettings,
        auth: AuthSettings,
        execution: ExecutionSettings,
    ) -> Self {
        Self {
            provider,
            workspace,
            auth,
            execution,
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.provider.provider_name.trim().is_empty() {
            return Err("provider name is required");
        }
        if self.provider.model.trim().is_empty() {
            return Err("model is required");
        }
        if self.workspace.name.trim().is_empty() {
            return Err("workspace name is required");
        }
        if self.workspace.root.as_os_str().is_empty() {
            return Err("workspace root is required");
        }
        if self.auth.token.trim().is_empty() {
            return Err("auth token is required");
        }
        Ok(())
    }

    pub fn to_app_config(&self) -> AppConfig {
        AppConfig::new(
            self.provider.clone(),
            self.workspace.clone(),
            self.auth.clone(),
            ManagedAssetsSettings::default(),
        )
    }
}

impl ExecutionSettings {
    pub fn local_default() -> Self {
        Self {
            mode: ExecutionMode::Local,
            backend: ExecutionBackendSelection::local_command(),
        }
    }

    pub fn sandboxed() -> Self {
        Self {
            mode: ExecutionMode::Sandboxed,
            backend: ExecutionBackendSelection::sandbox(),
        }
    }

    pub fn disabled() -> Self {
        Self {
            mode: ExecutionMode::Disabled,
            backend: ExecutionBackendSelection::disabled(),
        }
    }

    pub fn sandbox_policy() -> SandboxPolicySettings {
        SandboxPolicySettings {
            enabled: true,
            backend: ExecutionBackendSelection::sandbox(),
        }
    }

    pub fn execution_dir(home: impl AsRef<Path>) -> PathBuf {
        AppConfig::config_dir(home)
    }

    pub fn execution_path(home: impl AsRef<Path>) -> PathBuf {
        Self::execution_dir(home).join("execution.json")
    }

    pub fn save_to_home(&self, home: impl AsRef<Path>) -> io::Result<PathBuf> {
        let path = Self::execution_path(home);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialized = self.to_json_string()?;
        fs::write(&path, serialized)?;
        Ok(path)
    }

    pub fn to_json_string(&self) -> io::Result<String> {
        let body = serde_json::json!({
            "schemaVersion": "1",
            "mode": self.mode,
            "backend": self.backend,
        });
        serde_json::to_string_pretty(&body)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }

    pub fn load_from_home(home: impl AsRef<Path>) -> io::Result<Self> {
        let path = Self::execution_path(home);
        let body = fs::read_to_string(&path)?;
        #[derive(Deserialize)]
        struct ExecutionSettingsFile {
            #[serde(rename = "schemaVersion")]
            _schema_version: Option<String>,
            mode: ExecutionMode,
            backend: ExecutionBackendSelection,
        }

        let parsed: ExecutionSettingsFile = serde_json::from_str(&body)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        Ok(Self {
            mode: parsed.mode,
            backend: parsed.backend,
        })
    }
}

impl AppConfig {
    pub fn new(
        provider: ProviderSettings,
        workspace: WorkspaceSettings,
        auth: AuthSettings,
        managed_assets: ManagedAssetsSettings,
    ) -> Self {
        Self {
            provider,
            workspace,
            auth,
            managed_assets,
        }
    }

    pub fn config_dir(home: impl AsRef<Path>) -> PathBuf {
        home.as_ref().join(".matrixclaw").join("config")
    }

    pub fn config_path(home: impl AsRef<Path>) -> PathBuf {
        Self::config_dir(home).join("config.json")
    }

    pub fn default_first_launch(home: impl AsRef<Path>) -> Self {
        let home = home.as_ref();
        Self::new(
            ProviderSettings::new("openai-compatible", "gpt-5.4"),
            WorkspaceSettings::new("default", home.join("workspace")),
            AuthSettings::new("local-setup-token"),
            ManagedAssetsSettings::default(),
        )
    }

    pub fn save_to_home(&self, home: impl AsRef<Path>) -> io::Result<PathBuf> {
        let path = Self::config_path(home);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialized = self.to_json_string()?;
        fs::write(&path, serialized)?;
        Ok(path)
    }

    pub fn to_json_string(&self) -> io::Result<String> {
        let body = serde_json::json!({
            "schemaVersion": "1",
            "provider": self.provider,
            "workspace": self.workspace,
            "auth": self.auth,
            "managedAssets": self.managed_assets,
        });
        serde_json::to_string_pretty(&body)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }

    pub fn load_from_home(home: impl AsRef<Path>) -> io::Result<Self> {
        let path = Self::config_path(home);
        let body = fs::read_to_string(&path)?;
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct AppConfigFile {
            #[serde(rename = "schemaVersion")]
            _schema_version: Option<String>,
            provider: ProviderSettings,
            workspace: WorkspaceSettings,
            auth: AuthSettings,
            managed_assets: ManagedAssetsSettings,
        }

        let parsed: AppConfigFile = serde_json::from_str(&body)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        Ok(Self {
            provider: parsed.provider,
            workspace: parsed.workspace,
            auth: parsed.auth,
            managed_assets: parsed.managed_assets,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderSettings {
    pub provider_name: String,
    pub model: String,
}

impl ProviderSettings {
    pub fn new(provider_name: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            provider_name: provider_name.into(),
            model: model.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    pub name: String,
    pub root: PathBuf,
}

impl WorkspaceSettings {
    pub fn new(name: impl Into<String>, root: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            root: root.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSettings {
    pub token: String,
}

impl AuthSettings {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ManagedAssetsSettings {
    pub browser: BrowserAssetSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserAssetSettings {
    pub enabled: bool,
    pub version: String,
}

impl Default for BrowserAssetSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            version: String::new(),
        }
    }
}
