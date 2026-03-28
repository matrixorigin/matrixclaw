use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_agent_core::event::AgentEvent;
use matrixclaw_agent_core::message::ToolResultMessage;
use matrixclaw_agent_core::policy::{ToolPreflightDecision, ToolPreflightPolicy};
use matrixclaw_agent_core::provider::{Provider, ProviderError};
use matrixclaw_agent_core::r#loop::run_prompt_with_policy_trace_sink;
use matrixclaw_agent_core::tool::{ToolExecutionRequest, ToolExecutionResponse, ToolExecutor};
use matrixclaw_agent_core::{RunMessage, RunRequest};
use matrixclaw_session_runtime::queue::{QueueItem, SessionQueue};
use matrixclaw_session_runtime::recovery::{restore_session, SessionRecoveryStore};
use matrixclaw_session_runtime::session::Session;
use matrixclaw_session_runtime::sqlite::SqliteStorage;
use matrixclaw_session_runtime::RuntimeMessage;
use serde::{Deserialize, Serialize};

use crate::node::execution::ExecutionNodeCapabilityRequest;
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

#[derive(Debug, Clone)]
pub struct SessionBackedLiveRunService {
    home: PathBuf,
}

impl SessionBackedLiveRunService {
    pub fn new(home: impl AsRef<Path>) -> Self {
        Self {
            home: home.as_ref().to_path_buf(),
        }
    }

    pub fn run_with_provider(
        &self,
        model: impl Into<String>,
        request: LiveRunRequest,
        provider: &mut dyn Provider,
    ) -> Result<LiveRunOutcome, String> {
        self.run_with_provider_and_queue(model, request, None, provider)
    }

    pub fn run_with_provider_and_queue(
        &self,
        model: impl Into<String>,
        request: LiveRunRequest,
        bootstrap_queue: Option<SessionQueue>,
        provider: &mut dyn Provider,
    ) -> Result<LiveRunOutcome, String> {
        self.run_with_provider_and_queue_stream(
            model,
            request,
            bootstrap_queue,
            provider,
            &mut |_| {},
        )
    }

    pub fn run_with_provider_and_queue_stream(
        &self,
        model: impl Into<String>,
        request: LiveRunRequest,
        bootstrap_queue: Option<SessionQueue>,
        provider: &mut dyn Provider,
        on_event: &mut dyn FnMut(LiveRunEvent),
    ) -> Result<LiveRunOutcome, String> {
        let (session_id, mut session) =
            load_or_create_session_for_request(&self.home, request.session_id.as_deref())?;
        merge_bootstrap_queue(&mut session, bootstrap_queue.as_ref());

        let projection_kind = projection_kind_for_session(&session);
        let provider_prompt = build_provider_prompt(&session, projection_kind, &request.prompt);
        let context_messages = build_context_messages(&session, projection_kind, &request.prompt);
        let mut tool_executor = AppToolExecutor::new(self.home.clone());
        let mut policy = AppToolPolicy;
        let mut sequence = 0_u64;
        let mut on_agent_event = |event: AgentEvent| {
            on_event(LiveRunEvent::from_agent_event(sequence, event));
            sequence += 1;
        };
        let trace = run_prompt_with_policy_trace_sink(
            provider,
            &RunRequest {
                prompt: provider_prompt,
                context_messages,
            },
            Some(&mut tool_executor),
            Some(&mut policy),
            &mut on_agent_event,
        )
        .map_err(provider_error)?;

        finalize_session_after_run(
            &mut session,
            projection_kind,
            &request.prompt,
            &trace.events,
            &trace.result.final_message,
        );

        persist_session_for_id(&self.home, &session_id, &session)?;

        Ok(LiveRunOutcome {
            session_id,
            model: model.into(),
            streamed_message: trace.result.streamed_message,
            final_message: trace.result.final_message,
            events: trace
                .events
                .into_iter()
                .enumerate()
                .map(|(index, event)| LiveRunEvent::from_agent_event(index as u64, event))
                .collect(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromptProjectionKind {
    Direct,
    NextTurn,
    NextRun,
}

struct AppToolExecutor;
struct AppToolPolicy;

impl AppToolExecutor {
    fn new(_: PathBuf) -> Self {
        Self
    }

    fn execute_host_command(&self, request: &ToolExecutionRequest) -> ToolExecutionResponse {
        let response = ExecutionNodeCapabilityRequest::host_command_from_tool_arguments(
            &request.call.arguments,
            None,
        )
        .and_then(|request| request.execute())
        .and_then(|response| response.into_tool_output());

        match response {
            Ok(output) => ToolExecutionResponse::new(ToolResultMessage::new(
                request.call.tool_name.clone(),
                output,
            )),
            Err(error) => ToolExecutionResponse::new(ToolResultMessage::new(
                request.call.tool_name.clone(),
                error.to_string(),
            )),
        }
    }
}

impl ToolExecutor for AppToolExecutor {
    fn execute(&mut self, request: &ToolExecutionRequest) -> ToolExecutionResponse {
        match request.call.tool_name.as_str() {
            "add" => {
                let sum = request
                    .call
                    .arguments
                    .split(',')
                    .filter_map(|value| value.trim().parse::<i64>().ok())
                    .sum::<i64>();
                ToolExecutionResponse::new(ToolResultMessage::new("add", sum.to_string()))
            }
            "host.command" => self.execute_host_command(request),
            other => ToolExecutionResponse::new(ToolResultMessage::new(
                other,
                format!("unsupported tool: {other}"),
            )),
        }
    }
}

impl ToolPreflightPolicy for AppToolPolicy {
    fn before_tool_call(&mut self, request: &ToolExecutionRequest) -> ToolPreflightDecision {
        if request.call.tool_name == "danger" {
            return ToolPreflightDecision::Block(
                matrixclaw_agent_core::tool::BlockedToolResult::new(
                    request.call.clone(),
                    "policy denied execution",
                ),
            );
        }

        ToolPreflightDecision::Allow
    }
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
            AgentEvent::ToolCallStarted(content) => {
                ("tool_call_started".to_string(), Some(content))
            }
            AgentEvent::ToolExecutionStarted(content) => {
                ("tool_execution_started".to_string(), Some(content))
            }
            AgentEvent::ToolExecutionCompleted(content) => {
                ("tool_execution_completed".to_string(), Some(content))
            }
            AgentEvent::ToolResultAppended(content) => {
                ("tool_result_appended".to_string(), Some(content))
            }
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

fn build_provider_prompt(
    session: &Session,
    projection_kind: PromptProjectionKind,
    prompt: &str,
) -> String {
    let mut lines = match projection_kind {
        PromptProjectionKind::Direct => Vec::new(),
        PromptProjectionKind::NextTurn => session
            .project_next_turn()
            .into_iter()
            .map(render_runtime_message)
            .collect(),
        PromptProjectionKind::NextRun => session
            .project_next_run()
            .into_iter()
            .map(render_runtime_message)
            .collect(),
    };

    if matches!(projection_kind, PromptProjectionKind::Direct) {
        return prompt.to_string();
    }

    if !prompt.trim().is_empty() {
        lines.push(prompt.to_string());
    }

    lines.join("\n")
}

fn build_context_messages(
    session: &Session,
    projection_kind: PromptProjectionKind,
    prompt: &str,
) -> Vec<RunMessage> {
    let mut messages = match projection_kind {
        PromptProjectionKind::Direct => session
            .history()
            .iter()
            .map(run_message_from_runtime)
            .collect::<Vec<_>>(),
        PromptProjectionKind::NextTurn => session
            .project_next_turn()
            .into_iter()
            .map(|message| run_message_from_runtime(&message))
            .collect::<Vec<_>>(),
        PromptProjectionKind::NextRun => session
            .project_next_run()
            .into_iter()
            .map(|message| run_message_from_runtime(&message))
            .collect::<Vec<_>>(),
    };
    messages.push(RunMessage::user(prompt.to_string()));
    messages
}

fn run_message_from_runtime(message: &RuntimeMessage) -> RunMessage {
    match message {
        RuntimeMessage::User(content) => RunMessage::user(content.clone()),
        RuntimeMessage::Assistant(content) => RunMessage::assistant(content.clone()),
        RuntimeMessage::RuntimeSummary(content) => RunMessage::system(content.clone()),
        RuntimeMessage::ToolResult(content) => RunMessage::tool(content.clone()),
        RuntimeMessage::Steering(content) => RunMessage::system(content.clone()),
        RuntimeMessage::FollowUp(content) => RunMessage::system(content.clone()),
        RuntimeMessage::Warning(content) => RunMessage::system(content.clone()),
        RuntimeMessage::RetryMarker(content) => RunMessage::system(content.clone()),
    }
}

fn render_runtime_message(message: RuntimeMessage) -> String {
    match message {
        RuntimeMessage::User(content)
        | RuntimeMessage::Assistant(content)
        | RuntimeMessage::RuntimeSummary(content)
        | RuntimeMessage::ToolResult(content)
        | RuntimeMessage::Steering(content)
        | RuntimeMessage::FollowUp(content)
        | RuntimeMessage::Warning(content)
        | RuntimeMessage::RetryMarker(content) => content,
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
        if let AgentEvent::ToolResultAppended(content) = event {
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

fn provider_error(error: ProviderError) -> String {
    error.0
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
