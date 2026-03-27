use std::cell::Cell;

use matrixclaw_agent_core::event::AgentEvent;
use matrixclaw_agent_core::provider::{Provider, ProviderError};
use matrixclaw_agent_core::{run_prompt, RunRequest};

struct ProbeProvider {
    complete_calls: Cell<usize>,
    stream_calls: Cell<usize>,
}

impl ProbeProvider {
    fn new() -> Self {
        Self {
            complete_calls: Cell::new(0),
            stream_calls: Cell::new(0),
        }
    }
}

impl Provider for ProbeProvider {
    fn complete(&mut self, request: &RunRequest) -> Result<String, ProviderError> {
        self.complete_calls.set(self.complete_calls.get() + 1);
        Ok(format!("probe: {}", request.prompt))
    }

    fn stream(
        &mut self,
        request: &RunRequest,
        on_event: &mut dyn FnMut(AgentEvent),
    ) -> Result<String, ProviderError> {
        self.stream_calls.set(self.stream_calls.get() + 1);
        let streamed = format!("stream: {}", request.prompt);
        on_event(AgentEvent::MessageDelta(streamed.clone()));
        on_event(AgentEvent::MessageCompleted(streamed.clone()));
        Ok(streamed)
    }
}

#[test]
fn final_answer_generated_once() {
    let mut provider = ProbeProvider::new();
    let request = RunRequest::new("hello world");

    let result = run_prompt(&mut provider, &request).expect("run prompt");

    assert_eq!(
        provider.complete_calls.get(),
        0,
        "expected a single streamed generation without a probe completion"
    );
    assert_eq!(
        provider.stream_calls.get(),
        1,
        "expected exactly one streamed generation"
    );
    assert_eq!(
        result.streamed_message, result.final_message,
        "final persisted assistant message must match streamed content"
    );
}
