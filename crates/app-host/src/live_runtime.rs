use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_agent_core::event::AgentEvent;
use matrixclaw_agent_core::hooks::{CompositeHook, LifecycleHook};
use matrixclaw_agent_core::provider::Provider;
use matrixclaw_agent_core::r#loop::run_prompt_with_policy;
use matrixclaw_agent_core::{RunMessage, RunRequest, ToolChoice};
use matrixclaw_session_runtime::queue::{QueueItem, SessionQueue};
use matrixclaw_session_runtime::recovery::{restore_session, SessionRecoveryStore};
use matrixclaw_session_runtime::session::Session;
use matrixclaw_session_runtime::sqlite::SqliteStorage;
use matrixclaw_session_runtime::RuntimeMessage;
use matrixclaw_tools::builtin::delegate::{DelegateTool, SubagentRunner};
use matrixclaw_tools::builtin::delegate_parallel::{DelegateParallelTool, ParallelSubagentRunner};
use matrixclaw_tools::builtin::skill_evolver::{SkillEvolveTool, SkillRewriter};
use matrixclaw_tools::ToolRegistry;
use serde::{Deserialize, Serialize};

use crate::paths;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveRunRequest {
    pub prompt: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveRunEvent {
    pub sequence: u64,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveRunOutcome {
    pub session_id: String,
    pub model: String,
    pub streamed_message: String,
    pub final_message: String,
    pub events: Vec<LiveRunEvent>,
}

pub struct SessionBackedLiveRunService {
    home: PathBuf,
    registry: Arc<ToolRegistry>,
    hooks: tokio::sync::Mutex<CompositeHook>,
}

impl std::fmt::Debug for SessionBackedLiveRunService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionBackedLiveRunService")
            .field("home", &self.home)
            .field("registry", &self.registry)
            .finish_non_exhaustive()
    }
}

impl Clone for SessionBackedLiveRunService {
    fn clone(&self) -> Self {
        Self {
            home: self.home.clone(),
            registry: self.registry.clone(),
            hooks: tokio::sync::Mutex::new(CompositeHook::new()),
        }
    }
}

impl SessionBackedLiveRunService {
    pub async fn new(home: impl AsRef<Path>) -> Self {
        let registry = Arc::new(ToolRegistry::new());
        let workspace_root = home.as_ref().to_string_lossy().to_string();
        let tracker = Arc::new(matrixclaw_tools::SubagentTracker::new());
        matrixclaw_tools::builtin::register_all(&registry, &workspace_root, &tracker).await;

        let mcp_config_path = paths::config_dir(&home).join("mcp.json");
        let _report =
            matrixclaw_tools::mcp::registration::register_mcp_tools(&registry, &mcp_config_path)
                .await;

        Self {
            home: home.as_ref().to_path_buf(),
            registry,
            hooks: tokio::sync::Mutex::new(CompositeHook::new()),
        }
    }

    pub async fn tool_count(&self) -> usize {
        self.registry.list_descriptors().await.len()
    }

    pub async fn register_delegate_tool(&self, runner: SubagentRunner) {
        let tool = DelegateTool::new(runner, 0);
        self.registry.register(Arc::new(tool)).await;
    }

    pub async fn register_parallel_delegate_tool(&self, runner: ParallelSubagentRunner) {
        let tool = DelegateParallelTool::new(runner, 0);
        self.registry.register(Arc::new(tool)).await;
    }

    pub async fn add_hook(&self, hook: Box<dyn LifecycleHook>) {
        self.hooks.lock().await.add(hook);
    }

    pub async fn register_skill_evolve_tool(&self, rewriter: Arc<SkillRewriter>) {
        let tool = SkillEvolveTool::new(rewriter);
        self.registry.register(Arc::new(tool)).await;
    }

    pub fn registry(&self) -> Arc<ToolRegistry> {
        self.registry.clone()
    }

    pub async fn new_from_registry(home: impl AsRef<Path>, registry: Arc<ToolRegistry>) -> Self {
        Self {
            home: home.as_ref().to_path_buf(),
            registry,
            hooks: tokio::sync::Mutex::new(CompositeHook::new()),
        }
    }

    pub async fn run_with_provider(
        &self,
        model: impl Into<String>,
        request: LiveRunRequest,
        provider: &mut dyn Provider,
    ) -> Result<LiveRunOutcome, String> {
        self.run_with_provider_and_queue(model, request, None, provider)
            .await
    }

    pub async fn run_with_provider_and_queue(
        &self,
        model: impl Into<String>,
        request: LiveRunRequest,
        bootstrap_queue: Option<SessionQueue>,
        provider: &mut dyn Provider,
    ) -> Result<LiveRunOutcome, String> {
        let collected: Arc<Mutex<Vec<LiveRunEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let collector = collected.clone();
        let mut seq: u64 = 0;
        let mut on_agent_event = move |event: AgentEvent| {
            let live = LiveRunEvent::from_agent_event(seq, event);
            seq += 1;
            if let Ok(mut g) = collector.lock() {
                g.push(live);
            }
        };

        let outcome = self
            .run_inner(
                model,
                request,
                bootstrap_queue,
                provider,
                &mut on_agent_event,
            )
            .await?;

        let events = collected.lock().map(|g| g.clone()).unwrap_or_default();
        Ok(LiveRunOutcome {
            session_id: outcome.session_id,
            model: outcome.model,
            streamed_message: outcome.streamed_message,
            final_message: outcome.final_message,
            events,
        })
    }

    pub async fn run_with_provider_and_queue_stream(
        &self,
        model: impl Into<String>,
        request: LiveRunRequest,
        bootstrap_queue: Option<SessionQueue>,
        provider: &mut dyn Provider,
        on_event: &mut (dyn FnMut(LiveRunEvent) + Send),
    ) -> Result<LiveRunOutcome, String> {
        let mut seq: u64 = 0;
        let mut on_agent_event = |event: AgentEvent| {
            on_event(LiveRunEvent::from_agent_event(seq, event));
            seq += 1;
        };

        self.run_inner(
            model,
            request,
            bootstrap_queue,
            provider,
            &mut on_agent_event,
        )
        .await
    }

    async fn run_inner(
        &self,
        model: impl Into<String>,
        request: LiveRunRequest,
        bootstrap_queue: Option<SessionQueue>,
        provider: &mut dyn Provider,
        on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<LiveRunOutcome, String> {
        let (session_id, mut session) =
            load_or_create_session_for_request(&self.home, request.session_id.as_deref())?;
        merge_bootstrap_queue(&mut session, bootstrap_queue.as_ref());

        let projection_kind = projection_kind_for_session(&session);
        let context_messages = build_context_messages(&session, projection_kind, &request.prompt);
        let tool_descriptors = self.registry.list_descriptors().await;

        let run_request = RunRequest {
            prompt: String::new(),
            context_messages,
            tools: tool_descriptors,
            tool_choice: ToolChoice::Auto,
            max_iterations: 10,
        };

        let hooks_guard = self.hooks.lock().await;
        let hooks_ref = if hooks_guard.is_empty() {
            None
        } else {
            Some(&*hooks_guard)
        };
        let trace = run_prompt_with_policy(
            provider,
            &run_request,
            &self.registry,
            None,
            hooks_ref,
            on_event,
        )
        .await
        .map_err(|e| e.0.clone())?;
        drop(hooks_guard);

        finalize_session_after_run(
            &mut session,
            projection_kind,
            &request.prompt,
            &trace.events,
            &trace.result.final_message,
        );

        persist_session_for_id(&self.home, &session_id, &session)?;

        let events: Vec<LiveRunEvent> = trace
            .events
            .into_iter()
            .enumerate()
            .map(|(i, e)| LiveRunEvent::from_agent_event(i as u64, e))
            .collect();

        Ok(LiveRunOutcome {
            session_id,
            model: model.into(),
            streamed_message: trace.result.streamed_message,
            final_message: trace.result.final_message,
            events,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromptProjectionKind {
    Direct,
    NextTurn,
    NextRun,
}

impl LiveRunEvent {
    fn from_agent_event(sequence: u64, event: AgentEvent) -> Self {
        let (kind, content) = match event {
            AgentEvent::RunStarted => ("run_started".to_string(), None),
            AgentEvent::MessageStarted => ("message_started".to_string(), None),
            AgentEvent::MessageDelta(content) => ("message_delta".to_string(), Some(content)),
            AgentEvent::MessageCompleted(content) => {
                ("message_completed".to_string(), Some(content))
            }
            AgentEvent::ToolCallReceived(content) => {
                ("tool_call_started".to_string(), Some(content))
            }
            AgentEvent::ToolExecutionStarted(content) => {
                ("tool_execution_started".to_string(), Some(content))
            }
            AgentEvent::ToolExecutionCompleted(content) => {
                ("tool_execution_completed".to_string(), Some(content))
            }
            AgentEvent::ToolCallDelta {
                id,
                name,
                arguments_delta,
            } => (
                "tool_call_delta".to_string(),
                Some(format!("{id}:{name}:{arguments_delta}")),
            ),
            AgentEvent::IterationPressure { current, max, pct } => (
                "iteration_pressure".to_string(),
                Some(format!("iteration {current}/{max} ({pct}%)")),
            ),
            AgentEvent::RunCompleted => ("run_completed".to_string(), None),
        };

        Self {
            sequence,
            kind,
            content,
        }
    }
}

pub(crate) fn load_or_create_session(path: &Path) -> Result<Session, String> {
    if !path.exists() {
        return Ok(Session::new(Vec::new()));
    }

    let storage = SqliteStorage::open(path).map_err(|error| error.to_string())?;
    let snapshot = storage
        .load_recovery_snapshot()
        .map_err(|error| error.to_string())?;
    Ok(restore_session(snapshot).session)
}

fn projection_kind_for_session(session: &Session) -> PromptProjectionKind {
    let has_steering = session.queue().steering_items().next().is_some();
    let has_follow_up = session.queue().follow_up_items().next().is_some();

    if session.history().is_empty() && !has_steering && !has_follow_up {
        PromptProjectionKind::Direct
    } else if has_steering {
        PromptProjectionKind::NextTurn
    } else {
        PromptProjectionKind::NextRun
    }
}

fn build_context_messages(
    session: &Session,
    projection_kind: PromptProjectionKind,
    prompt: &str,
) -> Vec<RunMessage> {
    let mut messages: Vec<RunMessage> = match projection_kind {
        PromptProjectionKind::Direct => session
            .history()
            .iter()
            .map(run_message_from_runtime)
            .collect(),
        PromptProjectionKind::NextTurn => session
            .project_next_turn()
            .into_iter()
            .map(|m| run_message_from_runtime(&m))
            .collect(),
        PromptProjectionKind::NextRun => session
            .project_next_run()
            .into_iter()
            .map(|m| run_message_from_runtime(&m))
            .collect(),
    };
    messages.push(RunMessage::user(prompt.to_string()));
    messages
}

fn run_message_from_runtime(message: &RuntimeMessage) -> RunMessage {
    match message {
        RuntimeMessage::User(c) => RunMessage::user(c.clone()),
        RuntimeMessage::Assistant(c) => RunMessage::assistant(c.clone()),
        RuntimeMessage::RuntimeSummary(c) => RunMessage::system(c.clone()),
        RuntimeMessage::ToolResult(c) => RunMessage::tool_result("runtime", c.clone()),
        RuntimeMessage::Steering(c) => RunMessage::system(c.clone()),
        RuntimeMessage::FollowUp(c) => RunMessage::system(c.clone()),
        RuntimeMessage::Warning(c) => RunMessage::system(c.clone()),
        RuntimeMessage::RetryMarker(c) => RunMessage::system(c.clone()),
    }
}

fn finalize_session_after_run(
    session: &mut Session,
    projection_kind: PromptProjectionKind,
    prompt: &str,
    events: &[AgentEvent],
    final_message: &str,
) {
    match projection_kind {
        PromptProjectionKind::Direct => {}
        PromptProjectionKind::NextTurn => {
            let _ = session.drain_steering_messages();
        }
        PromptProjectionKind::NextRun => {
            let _ = session.drain_follow_up_messages();
        }
    }

    session
        .history_mut()
        .push(RuntimeMessage::User(prompt.to_string()));

    for event in events {
        if let AgentEvent::ToolExecutionCompleted(content) = event {
            session
                .history_mut()
                .push(RuntimeMessage::ToolResult(normalize_tool_result(content)));
        }
    }

    session
        .history_mut()
        .push(RuntimeMessage::Assistant(final_message.to_string()));
}

fn normalize_tool_result(content: &str) -> String {
    if content.starts_with("result:") {
        content.to_string()
    } else {
        format!("result:{content}")
    }
}

fn merge_bootstrap_queue(session: &mut Session, queue: Option<&SessionQueue>) {
    let Some(queue) = queue else {
        return;
    };

    for item in queue.items() {
        match item {
            QueueItem::Steering(message) => session.queue_steering_message(message.clone()),
            QueueItem::FollowUp(message) => session.queue_follow_up_message(message.clone()),
        }
    }
}

pub fn load_or_create_session_for_request(
    home: impl AsRef<Path>,
    session_id: Option<&str>,
) -> Result<(String, Session), String> {
    let session_id = session_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(generate_session_id);
    let session_path = session_db_path(home, &session_id);
    let session = load_or_create_session(&session_path)?;
    Ok((session_id, session))
}

pub fn persist_session_for_id(
    home: impl AsRef<Path>,
    session_id: &str,
    session: &Session,
) -> Result<(), String> {
    let session_path = session_db_path(home, session_id);
    persist_session(&session_path, session)
}

pub fn load_session_queue(
    home: impl AsRef<Path>,
    session_id: Option<&str>,
) -> Result<matrixclaw_session_runtime::queue::SessionQueue, String> {
    let Some(session_id) = session_id.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(matrixclaw_session_runtime::queue::SessionQueue::default());
    };

    let session_path = session_db_path(home, session_id);
    Ok(load_or_create_session(&session_path)?.queue().clone())
}

pub(crate) fn persist_session(path: &Path, session: &Session) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create session directory: {error}"))?;
    }

    let mut storage = SqliteStorage::open(path).map_err(|error| error.to_string())?;
    storage
        .persist_session(session)
        .map_err(|error| error.to_string())
}

fn generate_session_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    format!("session-{}-{}", std::process::id(), nanos)
}

pub fn session_db_path(home: impl AsRef<Path>, session_id: &str) -> PathBuf {
    paths::runtime_home(home)
        .join("state")
        .join("sessions")
        .join(format!("{}.sqlite3", session_file_stem(session_id)))
}

fn session_file_stem(session_id: &str) -> String {
    let trimmed = session_id.trim();
    let source = if trimmed.is_empty() {
        "session"
    } else {
        trimmed
    };
    let mut stem = String::with_capacity(source.len() * 2 + 3);
    stem.push_str("id-");
    for byte in source.as_bytes() {
        use std::fmt::Write as _;
        let _ = write!(&mut stem, "{byte:02x}");
    }
    stem
}
