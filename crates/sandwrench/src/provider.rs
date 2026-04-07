use crate::backend::daytona::DaytonaBackend;
use crate::backend::docker::DockerSandboxBackend;
use crate::backend::e2b::E2bBackend;
use crate::backend::local::LocalSandboxBackend;
#[cfg(feature = "ssh")]
use crate::backend::ssh::SshSandboxBackend;
use crate::config::{SandboxConfig, SandboxKind};
use crate::error::SandboxError;
use crate::runtime::{CodeRequest, CommandRequest, SandboxResult, SandboxRuntime};

pub struct SandboxProvider {
    config: SandboxConfig,
    runtime: Box<dyn SandboxRuntime>,
}

impl SandboxProvider {
    pub fn from_config(config: &SandboxConfig) -> Result<Self, SandboxError> {
        let runtime: Box<dyn SandboxRuntime> = match config.kind {
            SandboxKind::Docker => Box::new(DockerSandboxBackend::new(config.clone())?),
            SandboxKind::Local => Box::new(LocalSandboxBackend::new()),
            SandboxKind::E2b => Box::new(E2bBackend::new(config)?),
            SandboxKind::Daytona => Box::new(DaytonaBackend::new(config)?),
            SandboxKind::Wasm => {
                return Err(SandboxError::BackendUnavailable(
                    "WASM backend not yet implemented".into(),
                ))
            }
            #[cfg(feature = "ssh")]
            SandboxKind::Ssh => Box::new(SshSandboxBackend::new(config)?),
            #[cfg(not(feature = "ssh"))]
            SandboxKind::Ssh => {
                return Err(SandboxError::BackendUnavailable(
                    "SSH backend requires the 'ssh' feature flag".into(),
                ))
            }
        };
        Ok(Self {
            config: config.clone(),
            runtime,
        })
    }

    pub fn default_provider() -> Result<Self, SandboxError> {
        Self::from_config(&SandboxConfig::default())
    }

    pub async fn execute_code(&self, request: CodeRequest) -> Result<SandboxResult, SandboxError> {
        self.runtime.execute_code(request).await
    }

    pub async fn execute_command(
        &self,
        request: CommandRequest,
    ) -> Result<SandboxResult, SandboxError> {
        self.runtime.execute_command(request).await
    }

    pub fn kind(&self) -> SandboxKind {
        self.config.kind.clone()
    }

    pub fn is_available(&self) -> bool {
        self.runtime.is_available()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_provider_is_docker() {
        let provider = SandboxProvider::from_config(&SandboxConfig::default()).unwrap();
        assert_eq!(provider.kind(), SandboxKind::Docker);
    }

    #[test]
    fn local_provider() {
        let config = SandboxConfig {
            kind: SandboxKind::Local,
            ..SandboxConfig::default()
        };
        let provider = SandboxProvider::from_config(&config).unwrap();
        assert_eq!(provider.kind(), SandboxKind::Local);
        assert!(provider.is_available());
    }

    #[test]
    fn wasm_not_implemented() {
        let config = SandboxConfig {
            kind: SandboxKind::Wasm,
            ..SandboxConfig::default()
        };
        assert!(SandboxProvider::from_config(&config).is_err());
    }
}
