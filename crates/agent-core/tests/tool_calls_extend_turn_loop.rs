use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use matrixclaw_agent_core::event::AgentEvent;
use matrixclaw_agent_core::provider::{Provider, ProviderError, ProviderResponse};
use matrixclaw_agent_core::r#loop::run_prompt_with_policy;
use matrixclaw_agent_core::{RunRequest, ToolCall};
use matrixclaw_tools::{ToolDescriptor, ToolExecutor, ToolRegistry, ToolResult};
use std::sync::Arc;

struct ToolCallProvider {
    stream_calls: AtomicUsize,
}

impl ToolCallProvider {
    fn new() -> Self {
        Self {
            stream_calls: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl Provider for ToolCallProvider {
    async fn complete(&mut self, _request: &RunRequest) -> Result<ProviderResponse, ProviderError> {
        Ok(ProviderResponse::text("ignored"))
    }

    async fn stream(
        &mut self,
        _request: &RunRequest,
        on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<ProviderResponse, ProviderError> {
        let call_num = self.stream_calls.fetch_add(1, Ordering::SeqCst);

        if call_num == 0 {
            on_event(AgentEvent::MessageDelta("call:add(2,3)".to_string()));
            Ok(ProviderResponse::tool_calls(vec![ToolCall::new(
                "add_call_1".into(),
                "add".into(),
                serde_json::json!({"a": 2, "b": 3}),
            )]))
        } else {
            let msg = "result:5".to_string();
            on_event(AgentEvent::MessageDelta(msg.clone()));
            Ok(ProviderResponse::text(msg))
        }
    }
}

struct SumTool;

#[async_trait]
impl ToolExecutor for SumTool {
    fn descriptor(&self) -> &ToolDescriptor {
        static DESC: std::sync::OnceLock<ToolDescriptor> = std::sync::OnceLock::new();
        DESC.get_or_init(|| ToolDescriptor::new("add", "adds two numbers"))
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        assert_eq!(call.name, "add");
        ToolResult::success(&call, "5")
    }
}

#[tokio::test]
async fn tool_calls_extend_turn_loop() {
    let mut provider = ToolCallProvider::new();
    let request = RunRequest::new("add 2 and 3");
    let registry = ToolRegistry::new();
    registry.register(Arc::new(SumTool)).await;

    let trace = run_prompt_with_policy(&mut provider, &request, &registry, None, &mut |_| {})
        .await
        .expect("run prompt");

    assert_eq!(
        provider.stream_calls.load(Ordering::SeqCst),
        2,
        "expected a second turn after tool execution"
    );
    assert_eq!(
        trace.events,
        vec![
            AgentEvent::RunStarted,
            AgentEvent::MessageStarted,
            AgentEvent::MessageCompleted(String::new()),
            AgentEvent::ToolCallReceived("add".to_string()),
            AgentEvent::ToolExecutionStarted("add".to_string()),
            AgentEvent::ToolExecutionCompleted("5".to_string()),
            AgentEvent::MessageStarted,
            AgentEvent::MessageCompleted("result:5".to_string()),
            AgentEvent::RunCompleted,
        ],
        "expected ordered tool lifecycle and continuation"
    );
    assert_eq!(trace.result.final_message, "result:5");
}
