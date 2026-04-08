use std::env;
use std::sync::Arc;

use zstar_provider::config::load_or_default_config;
use zstar_provider::registry::ProviderRegistry;

use super::adapters::discord_stub::DiscordStubGateway;
use super::adapters::matrix_stub::MatrixStubGateway;
use super::adapters::slack_stub::SlackStubGateway;
use super::adapters::telegram_stub::TelegramStubGateway;
use super::agent_bridge::AgentBridge;
use super::config::GatewayConfig;
use super::platform::MessageGateway;
use crate::paths;

#[derive(Debug)]
pub struct GatewayArgs {
    pub platform: String,
    pub config_path: Option<String>,
}

pub fn parse_gateway_args(args: impl IntoIterator<Item = String>) -> Result<GatewayArgs, String> {
    let mut args = args.into_iter();
    let mut platform: Option<String> = None;
    let mut config_path: Option<String> = None;

    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--platform" => {
                let value = args
                    .next()
                    .ok_or("missing platform name after --platform")?;
                platform = Some(value);
            }
            "--config" => {
                let value = args.next().ok_or("missing config path after --config")?;
                config_path = Some(value);
            }
            other => return Err(format!("unknown gateway-serve option: {other}")),
        }
    }

    let platform = platform.ok_or("--platform is required")?;
    Ok(GatewayArgs {
        platform,
        config_path,
    })
}

pub async fn run_gateway_serve(args: GatewayArgs) -> Result<(), String> {
    let home = paths::home_dir();
    let config = GatewayConfig::load_or_default(&home);
    let model = config.resolve_model();

    let platform_config = config
        .platforms
        .get(&args.platform)
        .cloned()
        .ok_or_else(|| {
            format!(
                "platform '{}' not configured in gateway.json",
                args.platform
            )
        })?;

    let gateway: Box<dyn MessageGateway> = match args.platform.as_str() {
        "matrix" => {
            let adapter = MatrixStubGateway::from_config(&platform_config)?;
            Box::new(adapter)
        }
        "discord" => {
            let adapter = DiscordStubGateway::from_config(&platform_config)?;
            Box::new(adapter)
        }
        "telegram" => {
            let adapter = TelegramStubGateway::from_config(&platform_config)?;
            Box::new(adapter)
        }
        "slack" => {
            let adapter = SlackStubGateway::from_config(&platform_config)?;
            Box::new(adapter)
        }
        other => return Err(format!("unknown platform: {other}")),
    };

    let (provider_registry, fallback_chain) = build_provider_parts(&model).await?;

    let bridge = AgentBridge::new(
        provider_registry,
        fallback_chain,
        model,
        config.max_message_length,
    );

    gateway.start(Box::new(bridge)).await
}

async fn build_provider_parts(model: &str) -> Result<(Arc<ProviderRegistry>, Vec<String>), String> {
    let home = paths::home_dir();
    let config_path = paths::config_dir(&home).join("providers.json");
    let plane_config = load_or_default_config(Some(&config_path));

    if !plane_config.providers.is_empty() {
        let registry = Arc::new(ProviderRegistry::new());
        for pc in &plane_config.providers {
            registry.register(pc.clone()).await?;
        }
        return Ok((registry, plane_config.fallback_chain.clone()));
    }

    let api_key = env::var("OPENROUTER_API_KEY").map_err(|_| {
        "OPENROUTER_API_KEY is not set. Set it to an OpenRouter API key, or create ~/.zstar/config/providers.json".to_string()
    })?;

    use zstar_provider::backend::{ProviderConfig, ProviderType};

    let base_url = env::var("MATRIXCLAW_OPENAI_BASE_URL");
    let provider_type = if base_url.is_ok() {
        ProviderType::Custom
    } else {
        ProviderType::OpenAi
    };

    let config = ProviderConfig {
        name: "default".to_string(),
        provider_type,
        base_url: base_url.ok(),
        api_key: Some(api_key),
        models: vec![model.to_string()],
        rpm_limit: None,
    };

    let registry = Arc::new(ProviderRegistry::new());
    registry.register(config).await?;
    Ok((registry, vec!["default".to_string()]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_parsing_platform_and_config() {
        let args = vec![
            "--platform".to_string(),
            "matrix".to_string(),
            "--config".to_string(),
            "/tmp/gateway.json".to_string(),
        ];
        let parsed = parse_gateway_args(args).unwrap();
        assert_eq!(parsed.platform, "matrix");
        assert_eq!(parsed.config_path.as_deref(), Some("/tmp/gateway.json"));
    }

    #[test]
    fn args_parsing_requires_platform() {
        let args: Vec<String> = vec![];
        let result = parse_gateway_args(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--platform"));
    }

    #[test]
    fn args_parsing_unknown_flag() {
        let args = vec![
            "--platform".to_string(),
            "matrix".to_string(),
            "--bogus".to_string(),
        ];
        let result = parse_gateway_args(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown"));
    }
}
