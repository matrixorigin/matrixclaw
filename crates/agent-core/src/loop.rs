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
    let mut sink = |_| {};
    run_prompt_inner(provider, request, tool_executor, None, &mut sink)
}

pub fn run_prompt_with_policy_trace(
    provider: &mut dyn Provider,
    request: &RunRequest,
    tool_executor: Option<&mut dyn ToolExecutor>,
    policy: Option<&mut dyn ToolPreflightPolicy>,
) -> Result<RunTrace, ProviderError> {
    let mut sink = |_| {};
    run_prompt_inner(provider, request, tool_executor, policy, &mut sink)
}

pub fn run_prompt_with_policy_trace_sink(
    provider: &mut dyn Provider,
    request: &RunRequest,
    tool_executor: Option<&mut dyn ToolExecutor>,
    policy: Option<&mut dyn ToolPreflightPolicy>,
    on_event_sink: &mut dyn FnMut(AgentEvent),
) -> Result<RunTrace, ProviderError> {
    run_prompt_inner(provider, request, tool_executor, policy, on_event_sink)
}

fn run_prompt_inner(
    provider: &mut dyn Provider,
    request: &RunRequest,
    mut tool_executor: Option<&mut dyn ToolExecutor>,
    mut policy: Option<&mut dyn ToolPreflightPolicy>,
    on_event_sink: &mut dyn FnMut(AgentEvent),
) -> Result<RunTrace, ProviderError> {
    let mut events = Vec::new();
    let mut current_request = request.clone();

    let streamed_message = loop {
        let mut on_event = |event: AgentEvent| {
            emit_event(&mut events, on_event_sink, event);
        };

        let assistant_output = provider.stream(&current_request, &mut on_event)?;
        let tool_calls = parse_tool_calls(&assistant_output);

        if tool_calls.is_empty() {
            break assistant_output;
        }

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
                record_blocked_tool(&mut events, on_event_sink, &blocked);
                tool_results.push(blocked.result);
                continue;
            }

            let Some(tool_executor) = tool_executor.as_mut() else {
                saw_unhandled_tool = true;
                break;
            };

            emit_event(
                &mut events,
                on_event_sink,
                AgentEvent::ToolCallStarted(request.call.tool_name.clone()),
            );
            emit_event(
                &mut events,
                on_event_sink,
                AgentEvent::ToolExecutionStarted(request.call.tool_name.clone()),
            );

            let response = tool_executor.execute(&request);
            let result = response.result;

            emit_event(
                &mut events,
                on_event_sink,
                AgentEvent::ToolExecutionCompleted(result.output.clone()),
            );
            emit_event(
                &mut events,
                on_event_sink,
                AgentEvent::ToolResultAppended(result.output.clone()),
            );
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
        let mut saw_continuation_start = false;
        let mut on_continuation_event = |event: AgentEvent| {
            if !saw_continuation_start && matches!(event, AgentEvent::RunStarted) {
                saw_continuation_start = true;
                return;
            }
            saw_continuation_start = true;
            continuation_events.push(event);
        };
        let continuation_output = provider.stream(&current_request, &mut on_continuation_event)?;

        if parse_tool_calls(&continuation_output).is_empty() {
            for event in continuation_events {
                emit_event(&mut events, on_event_sink, event);
            }
            break continuation_output;
        }

        let synthetic_message = synthesize_tool_continuation(&tool_results);
        emit_event(&mut events, on_event_sink, AgentEvent::MessageStarted);
        emit_event(
            &mut events,
            on_event_sink,
            AgentEvent::MessageDelta(synthetic_message.clone()),
        );
        emit_event(
            &mut events,
            on_event_sink,
            AgentEvent::MessageCompleted(synthetic_message.clone()),
        );
        break synthetic_message;
    };

    emit_event(&mut events, on_event_sink, AgentEvent::RunCompleted);
    let final_message = streamed_message.clone();

    Ok(RunTrace {
        events,
        result: RunResult {
            streamed_message,
            final_message,
        },
    })
}

fn emit_event(
    events: &mut Vec<AgentEvent>,
    on_event_sink: &mut dyn FnMut(AgentEvent),
    event: AgentEvent,
) {
    on_event_sink(event.clone());
    events.push(event);
}

fn record_blocked_tool(
    events: &mut Vec<AgentEvent>,
    on_event_sink: &mut dyn FnMut(AgentEvent),
    blocked: &BlockedToolResult,
) {
    emit_event(
        events,
        on_event_sink,
        AgentEvent::ToolCallStarted(blocked.result.tool_name.clone()),
    );
    emit_event(
        events,
        on_event_sink,
        AgentEvent::ToolResultAppended(blocked.result.output.clone()),
    );
}
