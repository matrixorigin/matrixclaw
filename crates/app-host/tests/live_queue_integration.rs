use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use matrixclaw_agent_core::event::AgentEvent;
use matrixclaw_agent_core::provider::{Provider, ProviderError, ProviderResponse};
use matrixclaw_agent_core::{RunMessageRole, RunRequest};
use matrixclaw_app_host::live_runtime::{
    session_db_path, LiveRunRequest, SessionBackedLiveRunService,
};
use matrixclaw_session_runtime::queue::SessionQueue;
use matrixclaw_session_runtime::session::Session;
use matrixclaw_session_runtime::sqlite::SqliteStorage;
use matrixclaw_session_runtime::RuntimeMessage;

#[tokio::test]
async fn live_queue_integration() {
    let home = temp_home();
    let session_id = "live-queue-session";
    seed_session(
        &home,
        session_id,
        vec![
            RuntimeMessage::ToolResult("result:alpha".to_string()),
            RuntimeMessage::ToolResult("result:beta".to_string()),
        ],
        vec!["hold the line".to_string(), "what changed?".to_string()],
    );

    let service = SessionBackedLiveRunService::new(&home).await;
    let mut provider = RecordingProvider::default();

    let first = service
        .run_with_provider(
            "moonshotai/kimi-k2.5",
            LiveRunRequest {
                prompt: "start".to_string(),
                session_id: Some(session_id.to_string()),
            },
            &mut provider,
        )
        .await
        .expect("first live run");

    let second = service
        .run_with_provider(
            "moonshotai/kimi-k2.5",
            LiveRunRequest {
                prompt: "resume".to_string(),
                session_id: Some(session_id.to_string()),
            },
            &mut provider,
        )
        .await
        .expect("second live run");

    assert_eq!(
        provider.prompts.len(),
        2,
        "the live service should have issued one provider prompt per run"
    );
    assert_eq!(first.final_message, "Persisted hello");
    assert_eq!(second.final_message, "Persisted hello");

    assert_prompt_contains_in_order(
        &provider.prompts[0],
        &[
            "tool:result:alpha",
            "tool:result:beta",
            "system:hold the line",
            "assistant:next assistant turn",
            "user:start",
        ],
        "steering should appear before the next assistant turn and preserve prior tool-result ordering",
    );
    assert!(
        !provider.prompts[0].contains("what changed?"),
        "follow-up should not be delivered until the current run completes: {:?}",
        provider.prompts[0]
    );

    assert_prompt_contains_in_order(
        &provider.prompts[1],
        &[
            "tool:result:alpha",
            "tool:result:beta",
            "system:what changed?",
            "assistant:next run assistant turn",
            "user:resume",
        ],
        "follow-up should be deferred until the current run completes while preserving tool-result ordering",
    );
    assert!(
        !provider.prompts[1].contains("hold the line"),
        "steering should stay scoped to the next assistant turn: {:?}",
        provider.prompts[1]
    );
}

#[derive(Default)]
struct RecordingProvider {
    prompts: Vec<String>,
}

#[async_trait]
impl Provider for RecordingProvider {
    async fn complete(&mut self, _request: &RunRequest) -> Result<ProviderResponse, ProviderError> {
        Err(ProviderError(
            "recording provider only supports streamed live runs".to_string(),
        ))
    }

    async fn stream(
        &mut self,
        request: &RunRequest,
        on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<ProviderResponse, ProviderError> {
        let prompt = if request.context_messages.is_empty() {
            request.prompt.clone()
        } else {
            request
                .context_messages
                .iter()
                .map(|message| format!("{}:{}", role_name(&message.role), message.content))
                .collect::<Vec<_>>()
                .join("\n")
        };
        self.prompts.push(prompt);

        on_event(AgentEvent::MessageStarted);
        on_event(AgentEvent::MessageDelta("Persisted ".to_string()));
        on_event(AgentEvent::MessageDelta("hello".to_string()));
        on_event(AgentEvent::MessageCompleted("Persisted hello".to_string()));

        Ok(ProviderResponse::text("Persisted hello"))
    }
}

fn role_name(role: &RunMessageRole) -> &'static str {
    match role {
        RunMessageRole::User => "user",
        RunMessageRole::System => "system",
        RunMessageRole::Assistant => "assistant",
        RunMessageRole::Tool => "tool",
    }
}

fn assert_prompt_contains_in_order(prompt: &str, needles: &[&str], message: &str) {
    let mut cursor = 0;

    for needle in needles {
        let Some(relative) = prompt[cursor..].find(needle) else {
            panic!("{message}: missing {needle:?} after byte {cursor} in prompt {prompt:?}");
        };
        cursor += relative + needle.len();
    }
}

fn seed_session(
    home: &PathBuf,
    session_id: &str,
    history: Vec<RuntimeMessage>,
    steering_and_follow_up: Vec<String>,
) {
    let session_path = session_db_path(home, session_id);
    if let Some(parent) = session_path.parent() {
        fs::create_dir_all(parent).expect("create session directory");
    }

    let queue = SessionQueue::from_items(vec![
        matrixclaw_session_runtime::queue::QueueItem::Steering(steering_and_follow_up[0].clone()),
        matrixclaw_session_runtime::queue::QueueItem::FollowUp(steering_and_follow_up[1].clone()),
    ]);
    let session = Session::from_parts(history, queue, Vec::new());

    let mut storage = SqliteStorage::open(&session_path).expect("open session storage");
    storage
        .persist_session(&session)
        .expect("persist seeded session");
}

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();

    let home = std::env::temp_dir().join(format!(
        "matrixclaw-live-queue-home-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&home).expect("create temp home");
    home
}
