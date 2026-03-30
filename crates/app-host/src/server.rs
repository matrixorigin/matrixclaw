use std::fs;
use std::io::{self, Read};
use std::net::{SocketAddr, TcpListener};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, RecvError, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use matrixclaw_manifests::config::{
    AppConfig, AuthSettings, ExecutionSettings, ProviderSettings, WorkspaceSettings,
};
use matrixclaw_session_runtime::queue::SessionQueue;
use tiny_http::{Header, Method, Response, Server, StatusCode};

use crate::compat_registry::CompatRegistryEntry;
use crate::http::{HttpMethod, HttpRequest, SetupSurface};
use crate::paths;
use crate::ui_assets::UiAssetLayout;

const LOOPBACK_BIND: &str = "127.0.0.1:38495";

pub fn serve_for_home(home: impl AsRef<Path>) -> io::Result<()> {
    let surface = SetupSurface::new(home.as_ref(), UiAssetLayout::discover_for_home(&home));
    serve_surface(surface, LOOPBACK_BIND)
}

pub fn serve_with_demo_fixture(home: impl AsRef<Path>) -> io::Result<()> {
    let fixture = ensure_demo_fixture(home.as_ref())?;
    let surface = SetupSurface::with_state(
        home.as_ref(),
        UiAssetLayout::discover_for_home(&home),
        fixture.agent_name,
        fixture.queue,
    );
    serve_surface(surface, LOOPBACK_BIND)
}

pub fn serve_surface(surface: SetupSurface, bind_addr: &str) -> io::Result<()> {
    let server = bind_server(bind_addr)?;
    let local_addr = server.server_addr();
    println!("MatrixClaw listening at http://{local_addr}");
    run_server(server, surface, &never_shutdown())
}

pub struct TestServerHandle {
    pub address: SocketAddr,
    shutdown: Sender<()>,
    join_handle: Option<JoinHandle<io::Result<()>>>,
}

impl TestServerHandle {
    pub fn shutdown(mut self) -> io::Result<()> {
        let _ = self.shutdown.send(());
        if let Some(join_handle) = self.join_handle.take() {
            join_handle
                .join()
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "server thread panicked"))?
        } else {
            Ok(())
        }
    }
}

pub fn spawn_test_server(surface: SetupSurface) -> io::Result<TestServerHandle> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let server = Server::from_listener(listener, None)
        .map_err(|error| io::Error::new(io::ErrorKind::AddrInUse, error))?;
    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    let join_handle = thread::spawn(move || run_server(server, surface, &shutdown_rx));

    Ok(TestServerHandle {
        address,
        shutdown: shutdown_tx,
        join_handle: Some(join_handle),
    })
}

fn run_server(server: Server, surface: SetupSurface, shutdown_rx: &Receiver<()>) -> io::Result<()> {
    loop {
        if shutdown_rx.try_recv().is_ok() {
            return Ok(());
        }

        match server.recv_timeout(Duration::from_millis(100)) {
            Ok(Some(request)) => {
                if method_is_websocket_upgrade(&request)
                    && crate::http::openclaw_api::is_openclaw_websocket_route(request.url())
                {
                    let surface = surface.clone();
                    thread::spawn(move || {
                        let _ =
                            crate::http::openclaw_api::serve_openclaw_websocket(surface, request);
                    });
                    continue;
                }
                let response = map_request(&surface, request)?;
                let _ = response.0.respond(response.1);
            }
            Ok(None) => {}
            Err(error) => return Err(io::Error::new(io::ErrorKind::ConnectionAborted, error)),
        }
    }
}

fn method_is_websocket_upgrade(request: &tiny_http::Request) -> bool {
    request.method() == &Method::Get && crate::http::openclaw_api::is_websocket_upgrade(request)
}

fn map_request(
    surface: &SetupSurface,
    mut request: tiny_http::Request,
) -> io::Result<(tiny_http::Request, Response<Box<dyn Read + Send>>)> {
    let request_path = request.url().to_string();
    let request_origin = request_origin_header_value(&request).map(str::to_string);

    let method = match request.method() {
        Method::Get => HttpMethod::Get,
        Method::Post => HttpMethod::Post,
        _ => {
            let response = build_response(
                405,
                "text/plain; charset=utf-8",
                b"method not allowed".to_vec(),
            )?;
            return Ok((request, response));
        }
    };

    let mut body = Vec::new();
    request.as_reader().read_to_end(&mut body)?;

    if method == HttpMethod::Post
        && crate::http::agent_api::is_agent_run_stream_route(request.url())
    {
        let response = build_streaming_agent_response(surface.clone(), body)?;
        return Ok((request, response));
    }

    let response = surface.handle(HttpRequest {
        method,
        path: request_path.clone(),
        body,
    });

    let mut response = build_response(response.status_code, &response.content_type, response.body)?;
    if let Some(origin) = health_probe_origin_for_cors(&request_path, request_origin.as_deref()) {
        response = with_header(response, "Access-Control-Allow-Origin", origin)?;
        response = with_header(response, "Access-Control-Allow-Methods", "GET")?;
        response = with_header(response, "Vary", "Origin")?;
    }

    Ok((request, response))
}

fn build_streaming_agent_response(
    surface: SetupSurface,
    body: Vec<u8>,
) -> io::Result<Response<Box<dyn Read + Send>>> {
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    thread::spawn(move || {
        let mut writer = |chunk: Vec<u8>| -> io::Result<()> {
            tx.send(chunk)
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "stream receiver dropped"))
        };

        if let Err(error) = crate::http::agent_api::stream_agent_run(&surface, &body, &mut writer) {
            let _ = tx.send(crate::http::agent_api::sse_frame(
                &crate::http::agent_api::AgentRunStreamFrame::Error {
                    error: error.to_string(),
                },
            ));
        }
    });

    let content_type = Header::from_bytes("Content-Type", b"text/event-stream")
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid content type"))?;
    let cache_control = Header::from_bytes("Cache-Control", b"no-cache")
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cache control"))?;

    Ok(Response::new(
        StatusCode(200),
        vec![content_type, cache_control],
        Box::new(ChannelReader::new(rx)),
        None,
        None,
    ))
}

fn build_response(
    status_code: u16,
    content_type: &str,
    body: Vec<u8>,
) -> io::Result<Response<Box<dyn Read + Send>>> {
    let header = Header::from_bytes("Content-Type", content_type.as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid content type"))?;

    Ok(Response::new(
        StatusCode(status_code),
        vec![header],
        Box::new(std::io::Cursor::new(body.clone())),
        Some(body.len()),
        None,
    ))
}

fn with_header(
    response: Response<Box<dyn Read + Send>>,
    name: &str,
    value: &str,
) -> io::Result<Response<Box<dyn Read + Send>>> {
    let header = Header::from_bytes(name.as_bytes(), value.as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid header"))?;
    Ok(response.with_header(header))
}

fn request_origin_header_value(request: &tiny_http::Request) -> Option<&str> {
    request
        .headers()
        .iter()
        .find(|header| header.field.equiv("Origin"))
        .map(|header| header.value.as_str())
}

fn health_probe_origin_for_cors<'origin>(
    path: &str,
    origin: Option<&'origin str>,
) -> Option<&'origin str> {
    if !crate::http::routes::is_health_route(path) {
        return None;
    }

    let origin = origin?.trim();
    (!origin.is_empty()).then_some(origin)
}

fn bind_server(bind_addr: &str) -> io::Result<Server> {
    let listener = TcpListener::bind(bind_addr)?;
    Server::from_listener(listener, None)
        .map_err(|error| io::Error::new(io::ErrorKind::AddrInUse, error))
}

struct ChannelReader {
    rx: Receiver<Vec<u8>>,
    current: std::io::Cursor<Vec<u8>>,
    closed: bool,
}

impl ChannelReader {
    fn new(rx: Receiver<Vec<u8>>) -> Self {
        Self {
            rx,
            current: std::io::Cursor::new(Vec::new()),
            closed: false,
        }
    }
}

impl Read for ChannelReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let read = self.current.read(buf)?;
            if read > 0 {
                return Ok(read);
            }

            if self.closed {
                return Ok(0);
            }

            match self.rx.recv() {
                Ok(next) => {
                    self.current = std::io::Cursor::new(next);
                }
                Err(RecvError) => {
                    self.closed = true;
                }
            }
        }
    }
}

fn never_shutdown() -> Receiver<()> {
    let (_tx, rx) = mpsc::channel();
    rx
}

struct DemoFixture {
    agent_name: String,
    queue: SessionQueue,
}

fn ensure_demo_fixture(home: &Path) -> io::Result<DemoFixture> {
    let workspace_root = home.join("workspace");
    fs::create_dir_all(workspace_root.join("src"))?;
    fs::write(
        workspace_root.join("README.md"),
        "# MatrixClaw demo workspace\n",
    )?;
    fs::write(
        workspace_root.join("src").join("main.rs"),
        "fn main() {\n    println!(\"matrixclaw demo\");\n}\n",
    )?;
    fs::write(
        workspace_root.join("notes.md"),
        "Use [[workspace:README.md]] before editing the workspace source tree.\n",
    )?;

    let config = AppConfig::new(
        ProviderSettings::new("openai-compatible", "gpt-5.4"),
        WorkspaceSettings::new("default", workspace_root),
        AuthSettings::new("demo-token"),
        Default::default(),
    );
    let _ = config.save_to_home(home)?;
    let _ = ExecutionSettings::local_default().save_to_home(home)?;

    let runtime_home = paths::runtime_home(home);
    let skills_root = runtime_home.join("skills");
    let research_root = skills_root.join("research");
    let lint_root = skills_root.join("lint-bridge");
    let imports_root = home.join("imports");
    let research_import_root = imports_root.join("research");
    let lint_import_root = imports_root.join("lint-bridge");
    fs::create_dir_all(&research_root)?;
    fs::create_dir_all(&lint_root)?;
    fs::create_dir_all(&research_import_root)?;
    fs::create_dir_all(&lint_import_root)?;
    fs::write(research_root.join("SKILL.md"), "# Research\n")?;
    fs::write(lint_root.join("SKILL.md"), "# Lint Bridge\n")?;
    fs::write(
        research_root.join("matrixclaw.skill.json"),
        serde_json::json!({
            "name": "research",
            "kind": "skill",
            "runtime": "native"
        })
        .to_string(),
    )?;
    fs::write(
        lint_root.join("matrixclaw.skill.json"),
        serde_json::json!({
            "name": "lint-bridge",
            "kind": "skill",
            "runtime": "shimmed"
        })
        .to_string(),
    )?;
    fs::write(
        research_root.join("provenance.json"),
        serde_json::json!({
            "source": research_import_root,
            "imported_via": "demo-fixture"
        })
        .to_string(),
    )?;
    fs::write(
        lint_root.join("provenance.json"),
        serde_json::json!({
            "source": lint_import_root,
            "imported_via": "demo-fixture"
        })
        .to_string(),
    )?;

    let registry = vec![
        CompatRegistryEntry::from_skill_install(
            "research",
            research_import_root,
            &research_root,
            research_root.join("matrixclaw.skill.json"),
            research_root.join("provenance.json"),
        ),
        CompatRegistryEntry::from_skill_install(
            "lint-bridge",
            lint_import_root,
            &lint_root,
            lint_root.join("matrixclaw.skill.json"),
            lint_root.join("provenance.json"),
        ),
    ];

    let state_dir = runtime_home.join("state");
    fs::create_dir_all(&state_dir)?;
    fs::write(
        state_dir.join("compat-registry.json"),
        serde_json::to_string_pretty(&registry)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
    )?;

    let enabled_dir = runtime_home.join("agents").join("default");
    fs::create_dir_all(&enabled_dir)?;
    fs::write(
        enabled_dir.join("enabled-skills.json"),
        serde_json::json!({
            "schemaVersion": "1",
            "agentName": "default",
            "enabled": ["research"]
        })
        .to_string(),
    )?;

    let queue = SessionQueue::from_items(vec![
        matrixclaw_session_runtime::queue::QueueItem::Steering(
            "Prefer [[workspace:README.md]] before editing files.".to_string(),
        ),
        matrixclaw_session_runtime::queue::QueueItem::FollowUp(
            "After this run, enable lint-bridge for the default agent.".to_string(),
        ),
    ]);

    Ok(DemoFixture {
        agent_name: "default".to_string(),
        queue,
    })
}
