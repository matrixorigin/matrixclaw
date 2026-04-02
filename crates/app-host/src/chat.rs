use std::env;
use std::io::{self, Write};
use std::sync::Arc;

use matrixclaw_provider::backend::{ProviderConfig, ProviderType};
use matrixclaw_provider::config::load_or_default_config;
use matrixclaw_provider::fallback::FallbackProvider;
use matrixclaw_provider::registry::ProviderRegistry;

use crate::live_runtime::{LiveRunEvent, LiveRunRequest, SessionBackedLiveRunService};
use crate::paths;

const DEFAULT_MODEL: &str = "moonshotai/kimi-k2.5";

fn resolve_model(model_override: Option<&str>) -> String {
    if let Some(m) = model_override {
        return m.to_string();
    }
    env::var("MATRIXCLAW_LLM_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string())
}

async fn build_provider_from_env(api_key: &str, model: &str) -> Result<FallbackProvider, String> {
    let registry = ProviderRegistry::new();
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
        api_key: Some(api_key.to_string()),
        models: vec![model.to_string()],
        rpm_limit: None,
    };
    let registry = Arc::new(registry);
    registry.register(config).await?;
    Ok(FallbackProvider::new(registry, vec!["default".to_string()]))
}

pub async fn run_chat(model_override: Option<&str>) -> Result<(), String> {
    let model = resolve_model(model_override);
    let home = paths::home_dir();

    let config_path = paths::config_dir(&home).join("providers.json");
    let plane_config = load_or_default_config(Some(&config_path));

    let mut provider: FallbackProvider = if !plane_config.providers.is_empty() {
        let registry = Arc::new(ProviderRegistry::new());
        for pc in &plane_config.providers {
            registry.register(pc.clone()).await?;
        }
        FallbackProvider::new(registry, plane_config.fallback_chain.clone())
    } else {
        let api_key = env::var("OPENROUTER_API_KEY").map_err(|_| {
            "OPENROUTER_API_KEY is not set. Set it to an OpenRouter API key, or create ~/.matrixclaw/config/providers.json".to_string()
        })?;
        build_provider_from_env(&api_key, &model).await?
    };

    let service = SessionBackedLiveRunService::new(&home).await;
    let tool_count = service.tool_count().await;
    let mut session_id: Option<String> = None;

    println!("MatrixClaw chat — type your message and press Enter. Ctrl+C or /quit to exit.");
    println!("Model: {model} | Tools: {tool_count}");
    println!();

    loop {
        print!("> ");
        io::stdout()
            .flush()
            .map_err(|e| format!("flush failed: {e}"))?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("read failed: {e}"))?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }
        if input == "/quit" || input == "/exit" {
            println!("Goodbye.");
            return Ok(());
        }
        if input == "/clear" {
            session_id = None;
            println!("Session cleared.\n");
            continue;
        }
        if input == "/help" {
            print_help();
            continue;
        }
        if let Some(_model_arg) = input.strip_prefix("/model ") {
            println!("Note: /model switching is not supported with the provider control plane. Configure providers in providers.json.\n");
            continue;
        }

        let request = LiveRunRequest {
            prompt: input.to_string(),
            session_id: session_id.clone(),
        };

        let mut stdout = io::stdout();
        let mut on_event = |event: LiveRunEvent| match event.kind.as_str() {
            "message_delta" => {
                if let Some(content) = &event.content {
                    let _ = stdout.write_all(content.as_bytes());
                    let _ = stdout.flush();
                }
            }
            "tool_call_started" => {
                if let Some(content) = &event.content {
                    let _ = stdout.write_all(format!("\n  [tool call: {content}]\n").as_bytes());
                    let _ = stdout.flush();
                }
            }
            "tool_execution_completed" => {
                if let Some(content) = &event.content {
                    let preview = truncate_str(content, 120);
                    let _ = stdout.write_all(format!("  [tool result: {preview}]\n").as_bytes());
                    let _ = stdout.flush();
                }
            }
            "message_completed" => {
                let _ = stdout.write_all(b"\n\n");
                let _ = stdout.flush();
            }
            _ => {}
        };

        let result = service
            .run_with_provider_and_queue_stream(&model, request, None, &mut provider, &mut on_event)
            .await?;

        session_id = Some(result.session_id);
    }
}

fn print_help() {
    println!("Commands:");
    println!("  /help       Show this help");
    println!("  /quit       Exit chat");
    println!("  /exit       Exit chat");
    println!("  /clear      Clear session history");
    println!();
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.replace('\n', "\\n")
    } else {
        let truncated: String = s.chars().take(max_len).collect();
        format!("{}...", truncated.replace('\n', "\\n"))
    }
}
