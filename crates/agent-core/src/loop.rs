use matrixclaw_tools::ToolRegistry;

use crate::event::AgentEvent;
use crate::hooks::{CompositeHook, HookPayload};
use crate::message::RunMessage;
use crate::policy::{ToolPreflightDecision, ToolPreflightPolicy};
use crate::provider::{Provider, ProviderError};
use crate::{RunRequest, RunResult};

#[derive(Debug, Clone)]
pub struct RunTrace {
    pub events: Vec<AgentEvent>,
    pub result: RunResult,
}

pub async fn run_prompt(
    provider: &mut dyn Provider,
    request: &RunRequest,
    registry: &ToolRegistry,
    on_event: &mut (dyn FnMut(AgentEvent) + Send),
) -> Result<RunResult, ProviderError> {
    let trace = run_prompt_with_policy(provider, request, registry, None, None, on_event).await?;
    Ok(trace.result)
}

pub async fn run_prompt_with_policy(
    provider: &mut dyn Provider,
    request: &RunRequest,
    registry: &ToolRegistry,
    policy: Option<&dyn ToolPreflightPolicy>,
    hooks: Option<&CompositeHook>,
    on_event: &mut (dyn FnMut(AgentEvent) + Send),
) -> Result<RunTrace, ProviderError> {
    let mut events: Vec<AgentEvent> = Vec::new();
    let mut iterations: u32 = 0;
    let mut tool_calls_made: u32 = 0;
    let mut context = request.context_messages.clone();
    context.push(RunMessage::user(&request.prompt));

    emit(&mut events, on_event, AgentEvent::RunStarted);

    let final_message = loop {
        iterations += 1;

        if iterations == request.max_iterations * 70 / 100 {
            emit(
                &mut events,
                on_event,
                AgentEvent::IterationPressure {
                    current: iterations,
                    max: request.max_iterations,
                    pct: 70,
                },
            );
        }
        if iterations == request.max_iterations * 90 / 100 {
            emit(
                &mut events,
                on_event,
                AgentEvent::IterationPressure {
                    current: iterations,
                    max: request.max_iterations,
                    pct: 90,
                },
            );
        }

        if iterations > request.max_iterations {
            break "max iterations reached".to_string();
        }

        let current_request = RunRequest {
            prompt: String::new(),
            context_messages: context.clone(),
            tools: request.tools.clone(),
            tool_choice: request.tool_choice.clone(),
            max_iterations: request.max_iterations,
        };

        if let Some(hooks) = hooks {
            let payload = HookPayload::pre_llm_call(None, iterations);
            let action = hooks.dispatch(&payload).await;
            if action.block {
                let reason = action
                    .reason
                    .unwrap_or_else(|| "blocked by lifecycle hook".to_string());
                break reason;
            }
        }

        emit(&mut events, on_event, AgentEvent::MessageStarted);
        let response = provider.stream(&current_request, on_event).await?;
        emit(
            &mut events,
            on_event,
            AgentEvent::MessageCompleted(response.content.clone().unwrap_or_default()),
        );

        if let Some(hooks) = hooks {
            let payload = HookPayload::post_llm_call(
                None,
                iterations,
                response.content.as_deref().unwrap_or(""),
            );
            hooks.dispatch(&payload).await;
        }

        if !response.is_tool_call() {
            break response.content.unwrap_or_default();
        }

        context.push(RunMessage::assistant_with_tool_calls(
            response.content.clone().unwrap_or_default(),
            response.tool_calls.clone(),
        ));

        for call in &response.tool_calls {
            emit(
                &mut events,
                on_event,
                AgentEvent::ToolCallReceived(call.name.clone()),
            );

            if let Some(hooks) = hooks {
                let payload =
                    HookPayload::pre_tool_call(None, iterations, &call.name, &call.arguments);
                let action = hooks.dispatch(&payload).await;
                if action.block {
                    let reason = action
                        .reason
                        .unwrap_or_else(|| "blocked by lifecycle hook".to_string());
                    emit(
                        &mut events,
                        on_event,
                        AgentEvent::ToolExecutionCompleted(format!("hooked: {reason}")),
                    );
                    context.push(RunMessage::tool_result(
                        &call.id,
                        format!("hooked: {reason}"),
                    ));
                    continue;
                }
            }

            if let Some(policy) = policy {
                match policy.before_tool_call(call).await {
                    ToolPreflightDecision::Allow => {}
                    ToolPreflightDecision::Block(result) => {
                        emit(
                            &mut events,
                            on_event,
                            AgentEvent::ToolExecutionCompleted(format!(
                                "blocked: {}",
                                result.output
                            )),
                        );
                        context.push(RunMessage::tool_result(
                            &call.id,
                            format!("blocked: {}", result.output),
                        ));
                        continue;
                    }
                }
            }

            emit(
                &mut events,
                on_event,
                AgentEvent::ToolExecutionStarted(call.name.clone()),
            );
            let result = registry.execute(call.clone()).await;
            emit(
                &mut events,
                on_event,
                AgentEvent::ToolExecutionCompleted(result.output.clone()),
            );

            if let Some(hooks) = hooks {
                let payload =
                    HookPayload::post_tool_call(None, iterations, &call.name, &result.output);
                hooks.dispatch(&payload).await;
            }

            context.push(RunMessage::tool_result(&call.id, &result.output));
            tool_calls_made += 1;
        }
    };

    emit(&mut events, on_event, AgentEvent::RunCompleted);

    Ok(RunTrace {
        events,
        result: RunResult {
            streamed_message: final_message.clone(),
            final_message,
            tool_calls_made,
            iterations,
        },
    })
}

fn emit(
    events: &mut Vec<AgentEvent>,
    on_event: &mut (dyn FnMut(AgentEvent) + Send),
    event: AgentEvent,
) {
    on_event(event.clone());
    events.push(event);
}
