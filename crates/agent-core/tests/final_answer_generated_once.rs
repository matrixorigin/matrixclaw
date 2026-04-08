use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use zstar_agent_core::event::AgentEvent;
use zstar_agent_core::provider::{Provider, ProviderError, ProviderResponse};
use zstar_agent_core::{run_prompt, RunRequest};
use zstar_tools::ToolRegistry;

struct ProbeProvider {
    complete_calls: AtomicUsize,
    stream_calls: AtomicUsize,
}

impl ProbeProvider {
    fn new() -> Self {
        Self {
            complete_calls: AtomicUsize::new(0),
            stream_calls: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl Provider for ProbeProvider {
    async fn complete(&mut self, request: &RunRequest) -> Result<ProviderResponse, ProviderError> {
        self.complete_calls.fetch_add(1, Ordering::SeqCst);
        Ok(ProviderResponse::text(format!("probe: {}", request.prompt)))
    }

    async fn stream(
        &mut self,
        request: &RunRequest,
        on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<ProviderResponse, ProviderError> {
        self.stream_calls.fetch_add(1, Ordering::SeqCst);
        let streamed = format!("stream: {}", request.prompt);
        on_event(AgentEvent::MessageDelta(streamed.clone()));
        on_event(AgentEvent::MessageCompleted(streamed.clone()));
        Ok(ProviderResponse::text(streamed))
    }
}

#[tokio::test]
async fn final_answer_generated_once() {
    let mut provider = ProbeProvider::new();
    let request = RunRequest::new("hello world");
    let registry = ToolRegistry::new();

    let result = run_prompt(&mut provider, &request, &registry, &mut |_| {})
        .await
        .expect("run prompt");

    assert_eq!(
        provider.complete_calls.load(Ordering::SeqCst),
        0,
        "expected a single streamed generation without a probe completion"
    );
    assert_eq!(
        provider.stream_calls.load(Ordering::SeqCst),
        1,
        "expected exactly one streamed generation"
    );
    assert_eq!(
        result.streamed_message, result.final_message,
        "final persisted assistant message must match streamed content"
    );
}
