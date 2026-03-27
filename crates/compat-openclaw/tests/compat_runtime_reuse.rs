use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_compat_openclaw::http::openclaw_chat_http;
use matrixclaw_compat_openclaw::translation::{
    openclaw_session_db_path, OpenClawChatMessage, OpenClawChatRequest,
};
use matrixclaw_session_runtime::{
    ChatEvent, ChatInputMessage, ChatInputRole, ChatRequest, ChatRuntime,
};

#[derive(Default)]
struct RecordingRuntime {
    seen_requests: Vec<ChatRequest>,
}

impl ChatRuntime for RecordingRuntime {
    fn handle_chat(&mut self, request: ChatRequest) -> Vec<ChatEvent> {
        self.seen_requests.push(request);
        vec![
            ChatEvent::AssistantChunk("compatibility answer".to_string()),
            ChatEvent::Completed,
        ]
    }
}

#[test]
fn compat_runtime_reuse() {
    let _env_lock = env_lock().lock().expect("env lock");
    let conversation_id = format!(
        "compat-runtime-reuse-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos()
    );
    let home = temp_home();
    env::set_var("HOME", &home);
    let session_path = openclaw_session_db_path(&conversation_id);

    let request = OpenClawChatRequest::new(
        conversation_id.clone(),
        vec![
            OpenClawChatMessage::user("route me through the shared runtime"),
            OpenClawChatMessage::system("keep the boundary protocol-shaped"),
        ],
    );

    let mut runtime = RecordingRuntime::default();
    let response = openclaw_chat_http(&request, &mut runtime);

    assert_eq!(
        runtime.seen_requests,
        vec![ChatRequest {
            messages: vec![
                ChatInputMessage {
                    role: ChatInputRole::User,
                    content: "route me through the shared runtime".to_string(),
                },
                ChatInputMessage {
                    role: ChatInputRole::System,
                    content: "keep the boundary protocol-shaped".to_string(),
                },
            ],
        }],
        "the compatibility boundary should still translate OpenClaw chat into internal session-runtime messages"
    );

    assert_eq!(
        response.conversation_id, conversation_id,
        "the compatibility response should stay protocol-shaped"
    );
    assert_eq!(
        response.frames,
        vec![
            matrixclaw_compat_openclaw::stream_adapter::ChatFrame::AssistantChunk {
                content: "compatibility answer".to_string(),
            },
            matrixclaw_compat_openclaw::stream_adapter::ChatFrame::Completed,
        ],
        "the compatibility boundary should project runtime events back into protocol frames without browser-specific response types"
    );

    assert!(
        session_path.exists(),
        "OpenClaw chat should reuse the browser live runtime service and persist a session at {:?}",
        session_path
    );
    env::remove_var("HOME");
}

#[test]
fn compat_session_path_stays_inside_runtime_home() {
    let _env_lock = env_lock().lock().expect("env lock");
    let home = temp_home();
    env::set_var("HOME", &home);
    let session_path = openclaw_session_db_path("../../outside");

    assert!(
        session_path.starts_with(home.join(".matrixclaw").join("state").join("sessions")),
        "compat paths must stay inside the matrixclaw runtime root: {:?}",
        session_path
    );
    env::remove_var("HOME");
}

fn env_lock() -> &'static Mutex<()> {
    static ENV_LOCK: Mutex<()> = Mutex::new(());
    &ENV_LOCK
}

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!(
        "matrixclaw-compat-runtime-reuse-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&home).expect("create temp home");
    home
}
