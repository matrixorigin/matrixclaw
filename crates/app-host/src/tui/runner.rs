use std::env;
use std::sync::Arc;

use tokio::sync::Mutex;
use zstar_agent_core::nudge::NudgeEngine;
use zstar_provider::backend::{ProviderConfig, ProviderType};
use zstar_provider::config::load_or_default_config;
use zstar_provider::fallback::FallbackProvider;
use zstar_provider::registry::ProviderRegistry;
use zstar_tools::builtin::delegate::{SubagentRequest, SubagentResult};
use zstar_tools::builtin::delegate_parallel::ParallelSubagentRunner;
use zstar_tools::builtin::nudge_store::MemoryNudgeStore;
use zstar_tools::builtin::skill_evolver::{LlmRewriteFn, SkillRewriter};
use zstar_tools::builtin::skill_trace::{TraceCollector, TraceStore};

use crate::live_runtime::{LiveRunEvent, LiveRunRequest, SessionBackedLiveRunService};
use crate::paths;
use crate::tui::app::ChatApp;
use crate::tui::event::EventReader;
use crate::tui::theme::load_tui_config;

const DEFAULT_MODEL: &str = "moonshotai/kimi-k2.5";

fn resolve_model(model_override: Option<&str>) -> String {
    if let Some(m) = model_override {
        return m.to_string();
    }
    env::var("MATRIXCLAW_LLM_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string())
}

async fn build_provider_parts(
    api_key: &str,
    model: &str,
) -> Result<(Arc<ProviderRegistry>, Vec<String>), String> {
    let registry = ProviderRegistry::new();
    let base_url = env::var("ZSTAR_OPENAI_BASE_URL");
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
    Ok((registry, vec!["default".to_string()]))
}

fn make_subagent_runner(
    provider: Arc<Mutex<FallbackProvider>>,
    registry: Arc<zstar_tools::ToolRegistry>,
) -> zstar_tools::builtin::delegate::SubagentRunner {
    use zstar_agent_core::r#loop::run_prompt;
    use zstar_agent_core::{RunRequest, ToolChoice};
    use zstar_tools::builtin::delegate::SubagentRunner;

    Arc::new(move |req: SubagentRequest| {
        let provider = provider.clone();
        let registry = registry.clone();
        Box::pin(async move {
            let mut provider_guard = provider.lock().await;

            let prompt = if req.context.is_empty() {
                req.task.clone()
            } else {
                format!("{}\n\nAdditional context: {}", req.task, req.context)
            };

            let descriptors = registry.list_descriptors().await;
            let run_request = RunRequest {
                prompt,
                context_messages: Vec::new(),
                tools: descriptors,
                tool_choice: ToolChoice::Auto,
                max_iterations: 30,
            };

            match run_prompt(&mut *provider_guard, &run_request, &registry, &mut |_| {}).await {
                Ok(result) => SubagentResult {
                    final_message: result.final_message,
                    iterations: result.iterations,
                    tool_calls: result.tool_calls_made,
                    error: None,
                },
                Err(e) => SubagentResult {
                    final_message: String::new(),
                    iterations: 0,
                    tool_calls: 0,
                    error: Some(e.0),
                },
            }
        }) as std::pin::Pin<Box<dyn std::future::Future<Output = SubagentResult> + Send>>
    }) as SubagentRunner
}

fn make_parallel_subagent_runner(
    registry_ref: Arc<ProviderRegistry>,
    fallback_chain: Vec<String>,
    tool_registry: Arc<zstar_tools::ToolRegistry>,
) -> ParallelSubagentRunner {
    use zstar_agent_core::r#loop::run_prompt;
    use zstar_agent_core::{RunRequest, ToolChoice};

    Arc::new(move |requests: Vec<SubagentRequest>| {
        let registry = registry_ref.clone();
        let chain = fallback_chain.clone();
        let tools = tool_registry.clone();
        Box::pin(async move {
            let handles: Vec<_> = requests
                .into_iter()
                .map(|req| {
                    let reg = registry.clone();
                    let ch = chain.clone();
                    let t = tools.clone();
                    tokio::spawn(async move {
                        let mut provider = FallbackProvider::new(reg, ch);
                        let descriptors = t.list_descriptors().await;
                        let run_request = RunRequest {
                            prompt: if req.context.is_empty() {
                                req.task.clone()
                            } else {
                                format!("{}\n\nAdditional context: {}", req.task, req.context)
                            },
                            context_messages: Vec::new(),
                            tools: descriptors,
                            tool_choice: ToolChoice::Auto,
                            max_iterations: 30,
                        };
                        match run_prompt(&mut provider, &run_request, &t, &mut |_| {}).await {
                            Ok(result) => SubagentResult {
                                final_message: result.final_message,
                                iterations: result.iterations,
                                tool_calls: result.tool_calls_made,
                                error: None,
                            },
                            Err(e) => SubagentResult {
                                final_message: String::new(),
                                iterations: 0,
                                tool_calls: 0,
                                error: Some(e.0),
                            },
                        }
                    })
                })
                .collect();
            let mut results = Vec::with_capacity(handles.len());
            for handle in handles {
                match handle.await {
                    Ok(result) => results.push(result),
                    Err(e) => results.push(SubagentResult {
                        final_message: String::new(),
                        iterations: 0,
                        tool_calls: 0,
                        error: Some(format!("task panicked: {e}")),
                    }),
                }
            }
            results
        })
    })
}

fn make_llm_rewrite_fn(registry: Arc<ProviderRegistry>, chain: Vec<String>) -> LlmRewriteFn {
    use zstar_agent_core::provider::Provider;
    use zstar_agent_core::{RunRequest, ToolChoice};

    Box::new(move |prompt: String| {
        let registry = registry.clone();
        let chain = chain.clone();
        Box::pin(async move {
            let mut provider = FallbackProvider::new(registry, chain);
            let request = RunRequest {
                prompt,
                context_messages: Vec::new(),
                tools: Vec::new(),
                tool_choice: ToolChoice::None,
                max_iterations: 1,
            };
            match provider.complete(&request).await {
                Ok(response) => Ok(response.content.unwrap_or_default()),
                Err(e) => Err(e.0),
            }
        })
    })
}

pub async fn run_tui_chat(model_override: Option<&str>) -> Result<(), String> {
    let model = resolve_model(model_override);
    let home = paths::home_dir();
    let (_tui_config, theme) = load_tui_config(&home);

    let config_path = paths::config_dir(&home).join("providers.json");
    let plane_config = load_or_default_config(Some(&config_path));

    let (provider_arc, shared_registry, fallback_chain) = if !plane_config.providers.is_empty() {
        let registry = Arc::new(ProviderRegistry::new());
        for pc in &plane_config.providers {
            registry.register(pc.clone()).await?;
        }
        let chain = plane_config.fallback_chain.clone();
        let provider = Arc::new(Mutex::new(FallbackProvider::new(
            registry.clone(),
            chain.clone(),
        )));
        (provider, registry, chain)
    } else {
        let api_key = env::var("OPENROUTER_API_KEY").map_err(|_| {
                "OPENROUTER_API_KEY is not set. Set it to an OpenRouter API key, or create ~/.zstar/config/providers.json".to_string()
            })?;
        let (registry, chain) = build_provider_parts(&api_key, &model).await?;
        let provider = Arc::new(Mutex::new(FallbackProvider::new(
            registry.clone(),
            chain.clone(),
        )));
        (provider, registry, chain)
    };

    let router = plane_config.build_router(Some(model.clone()));

    let service = Arc::new(SessionBackedLiveRunService::new(&home).await);

    {
        let reg_clone = service.registry();
        let delegate_runner = make_subagent_runner(provider_arc.clone(), reg_clone.clone());
        service.register_delegate_tool(delegate_runner).await;
    }

    {
        let reg_clone = service.registry();
        let parallel_runner = make_parallel_subagent_runner(
            shared_registry.clone(),
            fallback_chain.clone(),
            reg_clone,
        );
        service
            .register_parallel_delegate_tool(parallel_runner)
            .await;
    }

    {
        let trace_db_path = TraceStore::db_path_for_home(&home);
        if let Ok(store) = TraceStore::open(&trace_db_path) {
            let collector = TraceCollector::new(store);
            service.add_hook(Box::new(collector)).await;
        }

        if let Ok(store) = TraceStore::open(&trace_db_path) {
            let skills_dir = home.join(".zstar").join("skills");
            let llm_call = make_llm_rewrite_fn(shared_registry.clone(), fallback_chain.clone());
            let rewriter = Arc::new(SkillRewriter::new(store, skills_dir, llm_call));
            service.register_skill_evolve_tool(rewriter).await;
        }
    }

    let tool_count = service.tool_count().await;

    let memory_db_path = MemoryNudgeStore::db_path_for_home(&home);
    let nudge_engine = MemoryNudgeStore::open(&memory_db_path)
        .ok()
        .map(|store| NudgeEngine::new(Box::new(store), 0.6, 3));

    crossterm::terminal::enable_raw_mode().map_err(|e| format!("raw mode: {e}"))?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)
        .map_err(|e| format!("alternate screen: {e}"))?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend).map_err(|e| format!("terminal: {e}"))?;
    terminal.clear().map_err(|e| format!("clear: {e}"))?;

    let (agent_tx, mut agent_rx) = tokio::sync::mpsc::channel::<LiveRunEvent>(256);
    let mut app = ChatApp::new(theme);
    let mut event_reader = EventReader::new();
    let session_id: Arc<tokio::sync::Mutex<Option<String>>> =
        Arc::new(tokio::sync::Mutex::new(None));

    let result = run_event_loop(
        &mut terminal,
        &mut agent_rx,
        &mut event_reader,
        &mut app,
        &service,
        &provider_arc,
        &router,
        &model,
        tool_count,
        &nudge_engine,
        &session_id,
        &agent_tx,
    )
    .await;

    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    );
    let _ = terminal.show_cursor();

    result
}

#[allow(clippy::too_many_arguments)]
async fn run_event_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    agent_rx: &mut tokio::sync::mpsc::Receiver<LiveRunEvent>,
    event_reader: &mut EventReader,
    app: &mut ChatApp,
    service: &Arc<SessionBackedLiveRunService>,
    provider_arc: &Arc<Mutex<FallbackProvider>>,
    router: &zstar_provider::router::ModelRouter,
    model: &str,
    tool_count: usize,
    nudge_engine: &Option<NudgeEngine>,
    session_id: &Arc<tokio::sync::Mutex<Option<String>>>,
    agent_tx: &tokio::sync::mpsc::Sender<LiveRunEvent>,
) -> Result<(), String> {
    loop {
        let event = event_reader.next(agent_rx).await;
        let user_text = app.handle_event(event);
        if app.should_quit {
            break;
        }
        if let Some(text) = user_text {
            if text == "/quit" || text == "/exit" {
                break;
            }
            if text == "/clear" {
                let mut sid = session_id.lock().await;
                *sid = None;
                app.responses.clear();
                continue;
            }

            let nudged = if let Some(ref engine) = nudge_engine {
                match engine.nudge(&text) {
                    Some(ctx) => format!("{ctx}\n\n{text}"),
                    None => text,
                }
            } else {
                text
            };

            let decision = router.route(&nudged, tool_count, &[]);
            let effective_model = decision.model.unwrap_or_else(|| model.to_string());

            let sid = session_id.lock().await.clone();
            let request = LiveRunRequest {
                prompt: nudged,
                session_id: sid,
            };

            let svc = Arc::clone(service);
            let provider = Arc::clone(provider_arc);
            let tx = agent_tx.clone();
            let sid_arc = Arc::clone(session_id);

            tokio::spawn(async move {
                let mut guard = provider.lock().await;
                let mut on_event = |event: LiveRunEvent| {
                    let _ = tx.blocking_send(event);
                };
                let result = svc
                    .run_with_provider_and_queue_stream(
                        effective_model,
                        request,
                        None,
                        &mut *guard,
                        &mut on_event,
                    )
                    .await;
                if let Ok(outcome) = result {
                    let mut sid = sid_arc.lock().await;
                    *sid = Some(outcome.session_id);
                }
            });
        }
        terminal
            .draw(|f| app.draw(f))
            .map_err(|e| format!("draw: {e}"))?;
    }

    Ok(())
}
