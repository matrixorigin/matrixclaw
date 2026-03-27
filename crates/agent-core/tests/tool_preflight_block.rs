use matrixclaw_agent_core::event::AgentEvent;
use matrixclaw_agent_core::message::{ToolCallMessage, ToolResultMessage};
use matrixclaw_agent_core::policy::{ToolPreflightDecision, ToolPreflightPolicy};
use matrixclaw_agent_core::provider::{Provider, ProviderError};
use matrixclaw_agent_core::r#loop::run_prompt_with_policy_trace;
use matrixclaw_agent_core::tool::{
    BlockedToolResult, ToolExecutionRequest, ToolExecutionResponse, ToolExecutor,
};
use matrixclaw_agent_core::RunRequest;

#[derive(Default)]
struct BlockedToolProvider {
    stream_calls: usize,
}

impl Provider for BlockedToolProvider {
    fn complete(&mut self, _request: &RunRequest) -> Result<String, ProviderError> {
        Ok("ignored".to_string())
    }

    fn stream(
        &mut self,
        _request: &RunRequest,
        on_event: &mut dyn FnMut(AgentEvent),
    ) -> Result<String, ProviderError> {
        self.stream_calls += 1;
        on_event(AgentEvent::RunStarted);
        on_event(AgentEvent::MessageStarted);
        on_event(AgentEvent::MessageDelta(
            "call:danger(delete_all)".to_string(),
        ));
        on_event(AgentEvent::MessageCompleted(
            "call:danger(delete_all)".to_string(),
        ));
        Ok("call:danger(delete_all)".to_string())
    }
}

#[derive(Default)]
struct DenyDangerPolicy {
    checks: usize,
}

impl ToolPreflightPolicy for DenyDangerPolicy {
    fn before_tool_call(&mut self, request: &ToolExecutionRequest) -> ToolPreflightDecision {
        self.checks += 1;
        assert_eq!(
            request.call,
            ToolCallMessage::new("danger", "delete_all"),
            "expected the policy to inspect the dangerous tool call"
        );

        ToolPreflightDecision::Block(BlockedToolResult::new(
            request.call.clone(),
            "policy denied execution",
        ))
    }
}

#[derive(Default)]
struct CountingTool {
    executions: usize,
}

impl ToolExecutor for CountingTool {
    fn execute(&mut self, request: &ToolExecutionRequest) -> ToolExecutionResponse {
        self.executions += 1;
        ToolExecutionResponse::new(ToolResultMessage::new(
            request.call.tool_name.clone(),
            "executed".to_string(),
        ))
    }
}

#[test]
fn tool_preflight_block() {
    let mut provider = BlockedToolProvider::default();
    let request = RunRequest {
        prompt: "delete everything".to_string(),
    };
    let mut policy = DenyDangerPolicy::default();
    let mut tool = CountingTool::default();

    let trace =
        run_prompt_with_policy_trace(&mut provider, &request, Some(&mut tool), Some(&mut policy))
            .expect("run prompt");

    assert_eq!(
        policy.checks, 1,
        "tool preflight should consult the blocking policy before execution"
    );
    assert_eq!(tool.executions, 0, "blocked tools must never execute");
    assert!(
        trace.events.contains(&AgentEvent::ToolResultAppended(
            "blocked: policy denied execution".to_string()
        )),
        "expected a structured blocked tool-result message"
    );
    assert_eq!(
        provider.stream_calls, 2,
        "the run should remain alive after the blocked tool call"
    );
    assert_eq!(trace.events.last(), Some(&AgentEvent::RunCompleted));
}
