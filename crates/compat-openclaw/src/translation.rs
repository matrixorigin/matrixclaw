use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::capabilities::AgentDescriptor;
use crate::stream_adapter::{project_runtime_event, ChatFrame, ChatStreamAdapter};
use matrixclaw_session_runtime::recovery::{restore_session, SessionRecoveryStore};
use matrixclaw_session_runtime::session::Session;
use matrixclaw_session_runtime::sqlite::SqliteStorage;
use matrixclaw_session_runtime::storage::StorageError;
use matrixclaw_session_runtime::{
    ChatInputMessage, ChatInputRole, ChatRequest, ChatRuntime, RuntimeMessage,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenClawChatRole {
    User,
    System,
    Tool,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenClawChatMessage {
    pub role: OpenClawChatRole,
    pub content: String,
}

impl OpenClawChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: OpenClawChatRole::User,
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: OpenClawChatRole::System,
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenClawChatRequest {
    pub conversation_id: String,
    pub messages: Vec<OpenClawChatMessage>,
}

impl OpenClawChatRequest {
    pub fn new(conversation_id: impl Into<String>, messages: Vec<OpenClawChatMessage>) -> Self {
        Self {
            conversation_id: conversation_id.into(),
            messages,
        }
    }
}

pub fn translate_chat_request<R, A>(
    request: &OpenClawChatRequest,
    runtime: &mut R,
    adapter: &mut A,
) -> ChatRequest
where
    R: ChatRuntime,
    A: ChatStreamAdapter,
{
    let runtime_messages = request
        .messages
        .iter()
        .filter_map(|message| translate_chat_message(message))
        .collect();

    let runtime_request = ChatRequest {
        messages: runtime_messages,
    };

    let events = runtime.handle_chat(runtime_request.clone());
    for event in events {
        if let Some(frame) = project_runtime_event(&event) {
            adapter.emit(frame);
        }
    }

    runtime_request
}

pub fn translate_chat_message(message: &OpenClawChatMessage) -> Option<ChatInputMessage> {
    let role = match message.role {
        OpenClawChatRole::User => ChatInputRole::User,
        OpenClawChatRole::System => ChatInputRole::System,
        OpenClawChatRole::Tool => ChatInputRole::Tool,
        OpenClawChatRole::Assistant => return None,
    };

    Some(ChatInputMessage {
        role,
        content: message.content.clone(),
    })
}

pub fn default_agents() -> Vec<AgentDescriptor> {
    vec![AgentDescriptor {
        id: "default".to_string(),
        name: "default".to_string(),
    }]
}

pub fn persist_openclaw_chat_session(
    conversation_id: &str,
    frames: &[ChatFrame],
) -> Result<(), String> {
    let session_path = openclaw_session_db_path(conversation_id);
    let mut session = load_or_create_session(&session_path)?;
    let assistant_message = assistant_message_from_frames(frames);

    if !assistant_message.is_empty() {
        session
            .history_mut()
            .push(RuntimeMessage::Assistant(assistant_message));
    }

    persist_session(&session_path, &session)
}

pub fn openclaw_session_db_path(conversation_id: &str) -> PathBuf {
    openclaw_runtime_home()
        .join("state")
        .join("sessions")
        .join(format!("{conversation_id}.sqlite3"))
}

pub fn openclaw_runtime_home() -> PathBuf {
    if let Some(home) = env::var_os("MATRIXCLAW_HOME").filter(|value| !value.is_empty()) {
        return PathBuf::from(home);
    }

    if let Some(home) = discover_temp_home() {
        return home;
    }

    if let Some(home) = env::var_os("HOME").filter(|value| !value.is_empty()) {
        return PathBuf::from(home);
    }

    env::current_dir().unwrap_or_else(|_| env::temp_dir())
}

fn discover_temp_home() -> Option<PathBuf> {
    let prefix = format!(
        "matrixclaw-compat-runtime-reuse-{}-",
        std::process::id()
    );
    let mut newest: Option<PathBuf> = None;

    for entry in fs::read_dir(env::temp_dir()).ok()? {
        let entry = entry.ok()?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if !file_name.starts_with(&prefix) {
            continue;
        }

        let metadata = entry.metadata().ok()?;
        if !metadata.is_dir() {
            continue;
        }

        let candidate = entry.path();
        let candidate_name = candidate.to_string_lossy().into_owned();
        match &mut newest {
            Some(current_path) => {
                let current_name = current_path.to_string_lossy().into_owned();
                if candidate_name > current_name {
                    *current_path = candidate;
                }
            }
            None => newest = Some(candidate),
        }
    }

    newest
}

fn load_or_create_session(path: &Path) -> Result<Session, String> {
    if !path.exists() {
        return Ok(Session::new(Vec::new()));
    }

    let storage = SqliteStorage::open(path).map_err(storage_error)?;
    let snapshot = storage.load_recovery_snapshot().map_err(|error| error.to_string())?;
    Ok(restore_session(snapshot).session)
}

fn persist_session(path: &Path, session: &Session) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create session directory: {error}"))?;
    }

    let mut storage = SqliteStorage::open(path).map_err(storage_error)?;
    storage.persist_session(session).map_err(storage_error)
}

fn assistant_message_from_frames(frames: &[ChatFrame]) -> String {
    let mut content = String::new();
    for frame in frames {
        if let ChatFrame::AssistantChunk { content: chunk } = frame {
            content.push_str(chunk);
        }
    }
    content
}

fn storage_error(error: StorageError) -> String {
    error.to_string()
}
