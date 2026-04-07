use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use russh::keys::PrivateKeyWithHashAlg;
use russh::{client, ChannelMsg};

use crate::config::{SandboxConfig, SandboxKind, SshAuthConfig};
use crate::error::SandboxError;
use crate::runtime::{CodeRequest, CommandRequest, SandboxResult, SandboxRuntime};

pub struct SshSandboxBackend {
    host: String,
    port: u16,
    username: String,
    auth: SshAuthConfig,
    working_dir: Option<String>,
    config: SandboxConfig,
}

struct SshClientHandler;

impl client::Handler for SshClientHandler {
    type Error = SandboxError;

    async fn check_server_key(
        &mut self,
        _key: &russh::keys::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

impl SshSandboxBackend {
    pub fn new(config: &SandboxConfig) -> Result<Self, SandboxError> {
        let ssh_config = config
            .ssh
            .as_ref()
            .ok_or_else(|| SandboxError::Config("ssh config required for SSH backend".into()))?;
        Ok(Self {
            host: ssh_config.host.clone(),
            port: ssh_config.port,
            username: ssh_config.username.clone(),
            auth: ssh_config.auth.clone(),
            working_dir: ssh_config.working_dir.clone(),
            config: config.clone(),
        })
    }

    async fn connect_and_auth(&self) -> Result<client::Handle<SshClientHandler>, SandboxError> {
        let config = Arc::new(client::Config::default());
        let mut handle = client::connect(config, (self.host.as_str(), self.port), SshClientHandler)
            .await
            .map_err(|e| SandboxError::SshConnection(e.to_string()))?;

        match &self.auth {
            SshAuthConfig::Password(password) => {
                let result = handle
                    .authenticate_password(&self.username, password.as_str())
                    .await
                    .map_err(|e| SandboxError::SshAuth(e.to_string()))?;
                if result != client::AuthResult::Success {
                    return Err(SandboxError::SshAuth("authentication rejected".into()));
                }
            }
            SshAuthConfig::Key { path, passphrase } => {
                let key = russh::keys::load_secret_key(path, passphrase.as_deref())
                    .map_err(|e| SandboxError::SshAuth(e.to_string()))?;
                let hash_alg = handle
                    .best_supported_rsa_hash()
                    .await
                    .map_err(|e| SandboxError::SshAuth(e.to_string()))?;
                let key_with_alg = PrivateKeyWithHashAlg::new(Arc::new(key), hash_alg.flatten());
                let result = handle
                    .authenticate_publickey(&self.username, key_with_alg)
                    .await
                    .map_err(|e| SandboxError::SshAuth(e.to_string()))?;
                if result != client::AuthResult::Success {
                    return Err(SandboxError::SshAuth("authentication rejected".into()));
                }
            }
            SshAuthConfig::Agent => {
                let sock = std::env::var("SSH_AUTH_SOCK")
                    .map_err(|_| SandboxError::SshAuth("SSH_AUTH_SOCK not set".into()))?;
                let stream = tokio::net::UnixStream::connect(&sock)
                    .await
                    .map_err(|e| SandboxError::SshAuth(format!("agent connect: {e}")))?;
                let mut agent = russh::keys::agent::client::AgentClient::connect(stream);
                let identities = agent
                    .request_identities()
                    .await
                    .map_err(|e| SandboxError::SshAuth(format!("agent identities: {e}")))?;
                let identity = identities
                    .into_iter()
                    .next()
                    .ok_or_else(|| SandboxError::SshAuth("no identities in SSH agent".into()))?;
                let public_key = match identity {
                    russh::keys::agent::AgentIdentity::PublicKey { key, .. } => key,
                    russh::keys::agent::AgentIdentity::Certificate { certificate, .. } => {
                        russh::keys::PublicKey::from(certificate.public_key().clone())
                    }
                };
                let result = handle
                    .authenticate_publickey_with(&self.username, public_key, None, &mut agent)
                    .await
                    .map_err(|e| SandboxError::SshAuth(e.to_string()))?;
                if result != client::AuthResult::Success {
                    return Err(SandboxError::SshAuth("authentication rejected".into()));
                }
            }
        }

        Ok(handle)
    }

    fn build_shell_command(&self, code: &str, language: &str) -> Result<String, SandboxError> {
        let escaped = code.replace('\'', "'\\''");
        match language {
            "python" | "py" => Ok(format!("echo '{}' | python3", escaped)),
            "rust" | "rs" => Ok(format!(
                "echo '{}' | rustc - -o /tmp/script && /tmp/script",
                escaped
            )),
            "javascript" | "js" => Ok(format!("echo '{}' | node", escaped)),
            "bash" | "sh" => Ok(escaped),
            _ => Err(SandboxError::UnsupportedLanguage(language.to_string())),
        }
    }

    async fn exec_remote(
        &self,
        command: &str,
        timeout_secs: u64,
    ) -> Result<SandboxResult, SandboxError> {
        let handle = self.connect_and_auth().await?;
        let mut channel = handle
            .channel_open_session()
            .await
            .map_err(|e| SandboxError::SshExec(e.to_string()))?;

        let full_command = match &self.working_dir {
            Some(dir) => format!("cd {} && {}", dir, command),
            None => command.to_string(),
        };

        channel
            .exec(true, &full_command as &str)
            .await
            .map_err(|e| SandboxError::SshExec(e.to_string()))?;

        let result = tokio::time::timeout(Duration::from_secs(timeout_secs), async {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let mut exit_code: i32 = -1;

            while let Some(msg) = channel.wait().await {
                match msg {
                    ChannelMsg::Data { data } => stdout.extend_from_slice(&data),
                    ChannelMsg::ExtendedData { data, ext: 1 } => stderr.extend_from_slice(&data),
                    ChannelMsg::ExtendedData { .. } => {}
                    ChannelMsg::ExitStatus { exit_status } => exit_code = exit_status as i32,
                    ChannelMsg::ExitSignal { error_message, .. } => {
                        stderr.extend_from_slice(error_message.as_bytes());
                        exit_code = -1;
                    }
                    ChannelMsg::Eof | ChannelMsg::Close => break,
                    _ => {}
                }
            }

            let stdout_str = String::from_utf8_lossy(&stdout).to_string();
            let stderr_str = String::from_utf8_lossy(&stderr).to_string();

            if exit_code != 0 {
                return SandboxResult::failure(exit_code, stderr_str, SandboxKind::Ssh);
            }

            SandboxResult::success(stdout_str, SandboxKind::Ssh)
        })
        .await;

        match result {
            Ok(output) => Ok(output),
            Err(_) => Ok(SandboxResult::timeout(timeout_secs, SandboxKind::Ssh)),
        }
    }
}

#[async_trait]
impl SandboxRuntime for SshSandboxBackend {
    fn kind(&self) -> SandboxKind {
        SandboxKind::Ssh
    }

    fn is_available(&self) -> bool {
        let Ok(mut addrs) = (self.host.as_str(), self.port).to_socket_addrs() else {
            return false;
        };
        let Some(addr) = addrs.next() else {
            return false;
        };
        std::net::TcpStream::connect_timeout(&addr, Duration::from_secs(2)).is_ok()
    }

    async fn execute_code(&self, request: CodeRequest) -> Result<SandboxResult, SandboxError> {
        let shell_cmd = self.build_shell_command(&request.code, &request.language)?;
        let timeout = request
            .timeout_secs
            .unwrap_or(self.config.default_timeout_secs);
        self.exec_remote(&shell_cmd, timeout).await
    }

    async fn execute_command(
        &self,
        request: CommandRequest,
    ) -> Result<SandboxResult, SandboxError> {
        let timeout = request
            .timeout_secs
            .unwrap_or(self.config.default_timeout_secs);
        let command = if request.args.is_empty() {
            request.command
        } else {
            format!("{} {}", request.command, request.args.join(" "))
        };
        let full_command = match request.cwd {
            Some(dir) => format!("cd {} && {}", dir, command),
            None => command,
        };
        self.exec_remote(&full_command, timeout).await
    }

    fn supported_languages(&self) -> Vec<String> {
        vec![
            "python".into(),
            "py".into(),
            "javascript".into(),
            "js".into(),
            "rust".into(),
            "rs".into(),
            "bash".into(),
            "sh".into(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::config::{SshAuthConfig, SshConfig};

    fn test_ssh_config() -> SandboxConfig {
        SandboxConfig {
            kind: SandboxKind::Ssh,
            ssh: Some(SshConfig {
                host: "unreachable.invalid".into(),
                port: 22,
                username: "test".into(),
                auth: SshAuthConfig::Agent,
                working_dir: Some("/tmp".into()),
            }),
            ..SandboxConfig::default()
        }
    }

    #[test]
    fn is_available_returns_false_for_unreachable() {
        let backend = SshSandboxBackend::new(&test_ssh_config()).unwrap();
        assert!(!backend.is_available());
    }

    #[test]
    fn build_shell_command_python() {
        let backend = SshSandboxBackend::new(&test_ssh_config()).unwrap();
        let cmd = backend
            .build_shell_command("print('hi')", "python")
            .unwrap();
        assert!(cmd.contains("python3"));
        assert!(cmd.contains("print"));
    }

    #[test]
    fn build_shell_command_javascript() {
        let backend = SshSandboxBackend::new(&test_ssh_config()).unwrap();
        let cmd = backend
            .build_shell_command("console.log('hi')", "js")
            .unwrap();
        assert!(cmd.contains("node"));
    }

    #[test]
    fn build_shell_command_bash() {
        let backend = SshSandboxBackend::new(&test_ssh_config()).unwrap();
        let cmd = backend.build_shell_command("echo hello", "bash").unwrap();
        assert!(cmd.contains("echo hello"));
    }

    #[test]
    fn build_shell_command_rust() {
        let backend = SshSandboxBackend::new(&test_ssh_config()).unwrap();
        let cmd = backend.build_shell_command("fn main() {}", "rust").unwrap();
        assert!(cmd.contains("rustc"));
    }

    #[test]
    fn build_shell_command_unsupported() {
        let backend = SshSandboxBackend::new(&test_ssh_config()).unwrap();
        assert!(backend.build_shell_command("code", "cobol").is_err());
    }

    #[test]
    fn build_shell_command_escapes_single_quotes() {
        let backend = SshSandboxBackend::new(&test_ssh_config()).unwrap();
        let cmd = backend.build_shell_command("it's a test", "bash").unwrap();
        assert!(cmd.contains("'\\''"));
    }

    #[test]
    fn supported_languages() {
        let backend = SshSandboxBackend::new(&test_ssh_config()).unwrap();
        let langs = backend.supported_languages();
        assert!(langs.contains(&"python".to_string()));
        assert!(langs.contains(&"javascript".to_string()));
        assert!(langs.contains(&"bash".to_string()));
    }

    #[test]
    fn kind_is_ssh() {
        let backend = SshSandboxBackend::new(&test_ssh_config()).unwrap();
        assert_eq!(backend.kind(), SandboxKind::Ssh);
    }

    #[test]
    fn requires_ssh_config() {
        let config = SandboxConfig {
            kind: SandboxKind::Ssh,
            ssh: None,
            ..SandboxConfig::default()
        };
        assert!(SshSandboxBackend::new(&config).is_err());
    }

    #[test]
    fn auth_config_serialization_agent() {
        let auth = SshAuthConfig::Agent;
        let json = serde_json::to_string(&auth).unwrap();
        assert_eq!(json, "\"agent\"");
        let deserialized: SshAuthConfig = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, SshAuthConfig::Agent));
    }

    #[test]
    fn auth_config_serialization_password() {
        let auth = SshAuthConfig::Password("secret".into());
        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("password"));
        let deserialized: SshAuthConfig = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, SshAuthConfig::Password(p) if p == "secret"));
    }

    #[test]
    fn auth_config_serialization_key() {
        let auth = SshAuthConfig::Key {
            path: PathBuf::from("/home/user/.ssh/id_rsa"),
            passphrase: Some("mypass".into()),
        };
        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("key"));
        assert!(json.contains("id_rsa"));
        let deserialized: SshAuthConfig = serde_json::from_str(&json).unwrap();
        match deserialized {
            SshAuthConfig::Key { path, passphrase } => {
                assert_eq!(path, PathBuf::from("/home/user/.ssh/id_rsa"));
                assert_eq!(passphrase, Some("mypass".into()));
            }
            _ => panic!("expected Key variant"),
        }
    }

    #[test]
    fn ssh_config_roundtrip() {
        let config = test_ssh_config();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SandboxConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.kind, SandboxKind::Ssh);
        assert!(deserialized.ssh.is_some());
        let ssh = deserialized.ssh.unwrap();
        assert_eq!(ssh.host, "unreachable.invalid");
        assert_eq!(ssh.port, 22);
        assert_eq!(ssh.username, "test");
        assert_eq!(ssh.working_dir, Some("/tmp".into()));
    }

    #[test]
    fn ssh_config_from_json() {
        let json = r#"{
            "kind": "ssh",
            "default_timeout_secs": 30,
            "memory_limit": "512m",
            "cpu_limit": 1.0,
            "network_enabled": false,
            "ssh": {
                "host": "sandbox.example.com",
                "port": 2222,
                "username": "sandbox",
                "auth": "agent",
                "working_dir": "/tmp/sandbox"
            }
        }"#;
        let config: SandboxConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.kind, SandboxKind::Ssh);
        let ssh = config.ssh.unwrap();
        assert_eq!(ssh.host, "sandbox.example.com");
        assert_eq!(ssh.port, 2222);
        assert_eq!(ssh.username, "sandbox");
        assert!(matches!(ssh.auth, SshAuthConfig::Agent));
        assert_eq!(ssh.working_dir, Some("/tmp/sandbox".into()));
    }

    #[test]
    fn ssh_config_default_port() {
        let json = r#"{
            "kind": "ssh",
            "default_timeout_secs": 30,
            "memory_limit": "512m",
            "cpu_limit": 1.0,
            "network_enabled": false,
            "ssh": {
                "host": "sandbox.example.com",
                "username": "sandbox",
                "auth": "agent"
            }
        }"#;
        let config: SandboxConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.ssh.unwrap().port, 22);
    }
}
