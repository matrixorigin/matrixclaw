use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use zstar_agent_core::event::AgentEvent;
use zstar_agent_core::policy::{ToolPreflightDecision, ToolPreflightPolicy};
use zstar_agent_core::provider::{Provider, ProviderError, ProviderResponse};
use zstar_agent_core::r#loop::run_prompt_with_policy;
use zstar_agent_core::{RunRequest, ToolCall, ToolResult};
use zstar_tools::ToolRegistry;

struct DenyDangerPolicy {
    checks: AtomicUsize,
}

impl DenyDangerPolicy {
    fn new() -> Self {
        Self {
            checks: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl ToolPreflightPolicy for DenyDangerPolicy {
    async fn before_tool_call(&self, call: &ToolCall) -> ToolPreflightDecision {
        self.checks.fetch_add(1, Ordering::SeqCst);
        assert_eq!(
            call.name, "danger",
            "expected the policy to inspect the dangerous tool call"
        );

        ToolPreflightDecision::Block(ToolResult::error(call, "policy denied execution"))
    }
}

struct BlockedToolProvider {
    stream_calls: AtomicUsize,
}

impl BlockedToolProvider {
    fn new() -> Self {
        Self {
            stream_calls: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl Provider for BlockedToolProvider {
    async fn complete(&mut self, _request: &RunRequest) -> Result<ProviderResponse, ProviderError> {
        Ok(ProviderResponse::text("ignored"))
    }

    async fn stream(
        &mut self,
        _request: &RunRequest,
        on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<ProviderResponse, ProviderError> {
        let call_num = self.stream_calls.fetch_add(1, Ordering::SeqCst);
        on_event(AgentEvent::MessageStarted);

        if call_num == 0 {
            let msg = "call:danger(delete_all)".to_string();
            on_event(AgentEvent::MessageDelta(msg.clone()));
            on_event(AgentEvent::MessageCompleted(msg));
            Ok(ProviderResponse::tool_calls(vec![ToolCall::new(
                "danger_call_1".into(),
                "danger".into(),
                serde_json::json!({"target": "delete_all"}),
            )]))
        } else {
            let msg = "no more tool calls".to_string();
            on_event(AgentEvent::MessageDelta(msg.clone()));
            on_event(AgentEvent::MessageCompleted(msg.clone()));
            Ok(ProviderResponse::text(msg))
        }
    }
}

#[tokio::test]
async fn tool_preflight_block() {
    let mut provider = BlockedToolProvider::new();
    let request = RunRequest::new("delete everything");
    let policy = DenyDangerPolicy::new();
    let registry = ToolRegistry::new();

    let trace = run_prompt_with_policy(
        &mut provider,
        &request,
        &registry,
        Some(&policy),
        None,
        &mut |_| {},
    )
    .await
    .expect("run prompt");

    assert_eq!(
        policy.checks.load(Ordering::SeqCst),
        1,
        "tool preflight should consult the blocking policy before execution"
    );
    assert!(
        trace.events.iter().any(
            |e| matches!(e, AgentEvent::ToolExecutionCompleted(msg) if msg.starts_with("blocked:"))
        ),
        "expected a structured blocked tool-result message"
    );
    assert_eq!(
        provider.stream_calls.load(Ordering::SeqCst),
        2,
        "the run should remain alive after the blocked tool call"
    );
    assert_eq!(trace.events.last(), Some(&AgentEvent::RunCompleted));
}
