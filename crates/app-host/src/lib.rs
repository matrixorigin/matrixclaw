pub mod agent_store;
pub mod asset_manifest;
pub mod assets;
pub mod chat;
pub mod commands;
pub mod compat_registry;
pub mod execution;
pub mod gateway;
pub mod http;
pub mod ingress;
pub mod install;
pub mod live_runtime;
pub mod llm_smoke;
pub mod local_command;
pub mod mcp_server;
pub mod node;
pub mod openai_compatible;
pub mod openclaw_transport;
pub mod paths;
pub mod plugin_launcher;
pub mod sandbox_backend;
pub mod server;
pub mod session_binding_store;
pub mod setup;
pub mod ui_assets;

pub const VERSION: &str = "0.1.0";

pub use ui_assets::{UiAssetKind, UiAssetLayout, UiResolvedAsset};

use std::sync::Arc;
use zstar_tools::ToolRegistry;

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime")
}

pub fn run(args: impl IntoIterator<Item = String>) -> i32 {
    let mut args = args.into_iter();
    let _ = args.next();
    match args.next().as_deref() {
        Some("version") => {
            println!("ZStar {VERSION}");
            0
        }
        Some("serve") => {
            let fixture = args.next();
            let home = paths::home_dir();
            let result = match fixture.as_deref() {
                Some("--fixture") => match args.next().as_deref() {
                    Some("demo") => server::serve_with_demo_fixture(&home),
                    Some(other) => Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("unknown fixture: {other}"),
                    )),
                    None => Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "missing fixture name after --fixture",
                    )),
                },
                Some(other) => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("unknown serve option: {other}"),
                )),
                None => server::serve_for_home(&home),
            };

            match result {
                Ok(()) => 0,
                Err(error) => {
                    eprintln!("server failed: {error}");
                    1
                }
            }
        }
        Some("llm-smoke") => {
            let mut model = "moonshotai/kimi-k2.5".to_string();
            while let Some(flag) = args.next() {
                match flag.as_str() {
                    "--model" => {
                        let Some(value) = args.next() else {
                            eprintln!("missing model name after --model");
                            return 1;
                        };
                        model = value;
                    }
                    other => {
                        eprintln!("unknown llm-smoke option: {other}");
                        return 1;
                    }
                }
            }

            let rt = runtime();
            match rt.block_on(llm_smoke::run_openrouter_smoke(&model)) {
                Ok(report) => {
                    println!("{report}");
                    0
                }
                Err(error) => {
                    eprintln!("llm smoke failed: {error}");
                    1
                }
            }
        }
        Some("chat") => {
            let mut model: Option<String> = None;
            while let Some(flag) = args.next() {
                match flag.as_str() {
                    "--model" => {
                        let Some(value) = args.next() else {
                            eprintln!("missing model name after --model");
                            return 1;
                        };
                        model = Some(value);
                    }
                    other => {
                        eprintln!("unknown chat option: {other}");
                        return 1;
                    }
                }
            }

            let rt = runtime();
            match rt.block_on(chat::run_chat(model.as_deref())) {
                Ok(()) => 0,
                Err(error) => {
                    eprintln!("chat failed: {error}");
                    1
                }
            }
        }
        Some("mcp-serve") => {
            let home = paths::home_dir();
            let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
            let registry = Arc::new(ToolRegistry::new());
            let tracker = Arc::new(zstar_tools::SubagentTracker::new());
            rt.block_on(zstar_tools::builtin::register_all(
                &registry,
                home.to_str().unwrap_or("."),
                &tracker,
            ));
            let server = mcp_server::McpServer::new(registry);
            match rt.block_on(server.run()) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("mcp server error: {e}");
                    1
                }
            }
        }
        Some("gateway-serve") => {
            let rt = runtime();
            match gateway::cli::parse_gateway_args(args) {
                Ok(gateway_args) => {
                    match rt.block_on(gateway::cli::run_gateway_serve(gateway_args)) {
                        Ok(()) => 0,
                        Err(error) => {
                            eprintln!("gateway failed: {error}");
                            1
                        }
                    }
                }
                Err(error) => {
                    eprintln!("gateway-serve: {error}");
                    1
                }
            }
        }
        None => match setup::ensure_first_launch() {
            Ok(setup::StartupMode::Ready) => 0,
            Ok(setup::StartupMode::Setup(surface)) => {
                println!("ZStar setup available at {}", surface.setup_url());
                println!("{}", gateway::matrix::matrix_gateway_status_message());
                0
            }
            Err(error) => {
                eprintln!("setup failed: {error}");
                1
            }
        },
        _ => {
            eprintln!(
                "usage: zstar version | zstar serve [--fixture demo] | zstar chat [--model <id>] | zstar llm-smoke [--model <id>] | zstar mcp-serve | zstar gateway-serve --platform <name> [--config <path>]"
            );
            1
        }
    }
}
