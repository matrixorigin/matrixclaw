use crate::event::AgentEvent;
use crate::policy::{ToolPreflightDecision, ToolPreflightPolicy};
use crate::provider::{Provider, ProviderError};
use crate::tool::{
    build_tool_result_prompt, parse_tool_calls, synthesize_tool_continuation, BlockedToolResult,
    ToolExecutionRequest, ToolExecutor,
};
use crate::{RunRequest, RunResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunTrace {
    pub events: Vec<AgentEvent>,
    pub result: RunResult,
}

pub fn run_prompt(
    provider: &mut dyn Provider,
    request: &RunRequest,
) -> Result<RunResult, ProviderError> {
    let trace = run_prompt_with_policy_trace(provider, request, None, None)?;
    Ok(trace.result)
}

pub fn run_prompt_with_trace(
    provider: &mut dyn Provider,
    request: &RunRequest,
    tool_executor: Option<&mut dyn ToolExecutor>,
) -> Result<RunTrace, ProviderError> {
    run_prompt_inner(provider, request, tool_executor, None)
}

pub fn run_prompt_with_policy_trace(
    provider: &mut dyn Provider,
    request: &RunRequest,
    tool_executor: Option<&mut dyn ToolExecutor>,
    policy: Option<&mut dyn ToolPreflightPolicy>,
) -> Result<RunTrace, ProviderError> {
    run_prompt_inner(provider, request, tool_executor, policy)
}

fn run_prompt_inner(
    provider: &mut dyn Provider,
    request: &RunRequest,
    mut tool_executor: Option<&mut dyn ToolExecutor>,
    mut policy: Option<&mut dyn ToolPreflightPolicy>,
) -> Result<RunTrace, ProviderError> {
    let mut events = Vec::new();
    let mut current_request = request.clone();

    let streamed_message = loop {
        let mut turn_events = Vec::new();
        let mut on_event = |event: AgentEvent| {
            turn_events.push(event);
        };

        let assistant_output = provider.stream(&current_request, &mut on_event)?;
        let tool_calls = parse_tool_calls(&assistant_output);

        if tool_calls.is_empty() {
            events.extend(turn_events);
            break assistant_output;
        }

        events.extend(turn_events);

        let mut tool_results = Vec::new();
        let mut saw_unhandled_tool = false;

        for call in tool_calls {
            let request = ToolExecutionRequest::new(call);
            let blocked_result = match policy.as_mut() {
                Some(policy) => match policy.before_tool_call(&request) {
                    ToolPreflightDecision::Allow => None,
                    ToolPreflightDecision::Block(blocked) => Some(blocked),
                },
                None => None,
            };

            if let Some(blocked) = blocked_result {
                record_blocked_tool(&mut events, &blocked);
                tool_results.push(blocked.result);
                continue;
            }

            let Some(tool_executor) = tool_executor.as_mut() else {
                saw_unhandled_tool = true;
                break;
            };

            events.push(AgentEvent::ToolCallStarted(request.call.tool_name.clone()));
            events.push(AgentEvent::ToolExecutionStarted(
                request.call.tool_name.clone(),
            ));

            let response = tool_executor.execute(&request);
            let result = response.result;

            events.push(AgentEvent::ToolExecutionCompleted(result.output.clone()));
            events.push(AgentEvent::ToolResultAppended(result.output.clone()));
            tool_results.push(result);
        }

        if saw_unhandled_tool {
            break assistant_output;
        }

        let continuation_prompt = build_tool_result_prompt(&tool_results);
        current_request = RunRequest {
            prompt: continuation_prompt.clone(),
            context_messages: Vec::new(),
        };

        let mut continuation_events = Vec::new();
        let mut on_continuation_event = |event: AgentEvent| {
            continuation_events.push(event);
        };
        let continuation_output = provider.stream(&current_request, &mut on_continuation_event)?;

        if parse_tool_calls(&continuation_output).is_empty() {
            if matches!(continuation_events.first(), Some(AgentEvent::RunStarted)) {
                continuation_events.remove(0);
            }
            events.extend(continuation_events);
            break continuation_output;
        }

        let synthetic_message = synthesize_tool_continuation(&tool_results);
        events.push(AgentEvent::MessageStarted);
        events.push(AgentEvent::MessageDelta(synthetic_message.clone()));
        events.push(AgentEvent::MessageCompleted(synthetic_message.clone()));
        break synthetic_message;
    };

    events.push(AgentEvent::RunCompleted);
    let final_message = streamed_message.clone();

    Ok(RunTrace {
        events,
        result: RunResult {
            streamed_message,
            final_message,
        },
    })
}

fn record_blocked_tool(events: &mut Vec<AgentEvent>, blocked: &BlockedToolResult) {
    events.push(AgentEvent::ToolCallStarted(
        blocked.result.tool_name.clone(),
    ));
    events.push(AgentEvent::ToolResultAppended(
        blocked.result.output.clone(),
    ));
}
