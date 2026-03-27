use matrixclaw_agent_core::event::AgentEvent;
use matrixclaw_agent_core::message::{ToolCallMessage, ToolResultMessage};
use matrixclaw_agent_core::provider::{Provider, ProviderError};
use matrixclaw_agent_core::r#loop::run_prompt_with_trace;
use matrixclaw_agent_core::tool::{ToolExecutionRequest, ToolExecutionResponse, ToolExecutor};
use matrixclaw_agent_core::RunRequest;

#[derive(Default)]
struct ToolCallProvider {
    stream_calls: usize,
}

impl Provider for ToolCallProvider {
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
        on_event(AgentEvent::MessageDelta("call:add(2,3)".to_string()));
        on_event(AgentEvent::MessageCompleted("call:add(2,3)".to_string()));
        Ok("call:add(2,3)".to_string())
    }
}

struct SumTool;

impl ToolExecutor for SumTool {
    fn execute(&mut self, request: &ToolExecutionRequest) -> ToolExecutionResponse {
        assert_eq!(request.call, ToolCallMessage::new("add", "2,3"));
        ToolExecutionResponse::new(ToolResultMessage::new("add", "5"))
    }
}

#[test]
fn tool_calls_extend_turn_loop() {
    let mut provider = ToolCallProvider::default();
    let request = RunRequest::new("add 2 and 3");
    let mut tool = SumTool;

    let trace =
        run_prompt_with_trace(&mut provider, &request, Some(&mut tool)).expect("run prompt");

    assert_eq!(
        provider.stream_calls, 2,
        "expected a second turn after tool execution"
    );
    assert_eq!(
        trace.events,
        vec![
            AgentEvent::RunStarted,
            AgentEvent::MessageStarted,
            AgentEvent::MessageDelta("call:add(2,3)".to_string()),
            AgentEvent::MessageCompleted("call:add(2,3)".to_string()),
            AgentEvent::ToolCallStarted("add".to_string()),
            AgentEvent::ToolExecutionStarted("add".to_string()),
            AgentEvent::ToolExecutionCompleted("5".to_string()),
            AgentEvent::ToolResultAppended("5".to_string()),
            AgentEvent::MessageStarted,
            AgentEvent::MessageDelta("result:5".to_string()),
            AgentEvent::MessageCompleted("result:5".to_string()),
            AgentEvent::RunCompleted,
        ],
        "expected ordered tool lifecycle and continuation"
    );
    assert_eq!(trace.result.final_message, "result:5");
}
