# Runtime Model

## Purpose

This document defines the runtime state model for MatrixClaw.

It answers the questions that usually become bugs later:

- what is a run
- what is a turn
- when is a message considered durable
- when do queued messages enter context
- who owns retry and compaction
- how streamed output maps to persisted output

The design goal is to preserve the strong loop shape learned from `pi-mono` while avoiding the persistence drift and double-generation issues seen in FastClaw.

## Core Runtime Vocabulary

### Session

A durable conversation container with:

- message history
- queue state
- workspace association
- provider/model policy
- retry and compaction metadata

### Run

A single execution attempt initiated by:

- a user prompt
- a queued steering message
- a queued follow-up message
- an internal continue after compaction or retry

A run may contain multiple turns.

### Turn

One assistant generation cycle plus any tool work triggered by that generation.

A turn starts when the provider begins producing an assistant message.
A turn ends when:

- the assistant finalizes without tool calls
- tool results are appended and the runtime decides to continue into another turn
- the run fails terminally
- the run is cancelled

### Queue Item

A deferred inbound message held by `session-runtime` until a policy-defined insertion point.

Initial queue kinds:

- `steering`
  - inject before the next assistant turn in the current run
- `follow_up`
  - inject only when the run would otherwise stop
- `system_runtime`
  - internal message produced by retry/compaction/session control logic

### Active Context

The in-memory message set used for the next provider call.

It is derived from:

- system/runtime messages
- persisted session messages
- queue items chosen for delivery
- optional compaction summary artifacts

### Durable Transcript

The user-visible conversation history stored by `session-runtime`.

Rule:

- if a user could reasonably quote it as part of the conversation, it belongs in the durable transcript

That includes:

- user messages
- assistant messages
- tool request/result messages that affect meaning
- assistant-visible runtime warnings shown to the user

## Message Model

MatrixClaw should keep a richer internal message model than provider wire roles.

## `AgentMessage`

Suggested conceptual variants:

- `System`
- `Developer`
- `RuntimeInstruction`
- `User`
- `Assistant`
- `ToolCall`
- `ToolResult`
- `RuntimeSummary`
- `Warning`
- `Error`

Important rules:

- `RuntimeSummary` is never persisted as a `User` message
- `ToolCall` and `ToolResult` are first-class messages with identifiers
- `Warning` and `Error` become transcript messages if surfaced to the user

## Message lifecycle

### User message

1. accepted by `app-host` or compatibility layer
2. persisted immediately by `session-runtime`
3. inserted into active context
4. associated with a new run

### Assistant message

1. created in-memory at `message_start`
2. receives deltas during provider streaming
3. finalized at `message_end`
4. persisted exactly once from the finalized content
5. exposed to all clients from the same finalized object

### Tool call message

1. derived from finalized assistant output
2. persisted before execution begins
3. used as the source of tool preflight

### Tool result message

1. created from structured execution output
2. persisted after execution completes or is blocked
3. inserted into active context before the next turn

## Run State Machine

Suggested high-level run states:

- `pending`
- `starting`
- `streaming_assistant`
- `awaiting_tool_preflight`
- `executing_tools`
- `awaiting_continuation`
- `retry_scheduled`
- `completed`
- `failed`
- `cancelled`

## Transition rules

### `pending -> starting`

Triggered when `session-runtime` decides to execute a run.

Preconditions:

- session lock acquired
- provider/model resolved
- active context materialized

### `starting -> streaming_assistant`

Triggered when `agent-core` begins provider streaming.

### `streaming_assistant -> awaiting_tool_preflight`

Triggered when the assistant message finalizes with one or more tool calls.

### `streaming_assistant -> completed`

Triggered when the assistant message finalizes with no further tool calls and no deliverable queue items remain.

### `awaiting_tool_preflight -> executing_tools`

Triggered when at least one tool call is allowed to run.

### `awaiting_tool_preflight -> awaiting_continuation`

Triggered when all tool calls are blocked but the assistant should still continue using the resulting tool result messages.

### `executing_tools -> awaiting_continuation`

Triggered when tool execution ends and results are persisted.

### `awaiting_continuation -> streaming_assistant`

Triggered when the runtime injects new context and decides another turn is required.

### `any active state -> retry_scheduled`

Triggered on classified retryable failure owned by `session-runtime`.

### `any active state -> failed`

Triggered on terminal error.

### `any active state -> cancelled`

Triggered when operator or client cancellation succeeds.

## Event Stream Contract

The event stream is the canonical integration boundary across runtime layers.

Suggested event families:

- lifecycle
- message
- tool
- queue
- persistence
- warning
- metrics

## Required event types

### Lifecycle

- `run_started`
- `turn_started`
- `turn_completed`
- `run_completed`
- `run_failed`
- `run_cancelled`

### Message

- `message_started`
- `message_delta`
- `message_completed`
- `message_persisted`

### Tool

- `tool_preflight_started`
- `tool_preflight_blocked`
- `tool_execution_started`
- `tool_execution_progress`
- `tool_execution_completed`
- `tool_result_persisted`

### Queue

- `queue_item_added`
- `queue_item_delivered`
- `queue_item_deferred`

### Runtime control

- `compaction_started`
- `compaction_completed`
- `retry_scheduled`
- `retry_started`

### Warning and diagnostics

- `runtime_warning`
- `provider_warning`
- `compat_warning`

## Event ordering rules

- `message_completed` must occur before any tool preflight for that assistant message
- `message_persisted` must reflect the same finalized assistant content that clients saw streamed
- `tool_result_persisted` must occur before the next assistant turn begins
- `run_completed` must not fire until all transcript-affecting persistence work is done

## Queue Delivery Semantics

Queue handling should be explicit and testable.

## Steering messages

Use case:

- operator correction during an active multi-turn run

Delivery rule:

- deliver before the next assistant turn begins
- never splice into a currently streaming assistant message
- preserve prior tool results and assistant output ordering

## Follow-up messages

Use case:

- user wants to ask something next without interrupting the current run

Delivery rule:

- hold until the run would otherwise complete
- start a fresh run after the current run reaches a stable stop point

## Runtime messages

Use case:

- internal retry notice
- overflow handling notice
- compaction summary marker

Delivery rule:

- injected only by `session-runtime`
- visible to the assistant according to policy
- persisted only when they change transcript meaning

## Tool Execution Semantics

## Preflight

Each tool call passes through:

1. existence check
2. argument schema validation
3. permission and policy evaluation
4. optional human approval policy in future modes

A blocked tool call must still produce:

- a structured result
- a persisted tool result message
- a visible reason code

## Execution

Initial execution policy:

- sequential by default
- optional bounded parallel groups for independent tools

Required execution properties:

- timeout support
- cancellation support
- stdout/stderr capture where relevant
- explicit exit status
- stable result identifiers

## Result insertion

Even if tools run in parallel, result messages should be inserted into the transcript in deterministic assistant source order unless a plugin contract explicitly requires a different ordering.

## Retry And Failure Ownership

`agent-core` should classify failures but not own retry policy.

## Failure taxonomy

Suggested classes:

- `provider_overflow`
- `provider_rate_limited`
- `provider_transient`
- `provider_auth`
- `tool_validation`
- `tool_runtime`
- `storage_transient`
- `compat_translation`
- `operator_cancelled`
- `unknown`

## Retry policy

Owned by `session-runtime`.

Initial policy:

- retry `provider_transient` with bounded exponential backoff
- compact and continue for `provider_overflow`
- do not retry `provider_auth` without operator intervention
- do not silently retry deterministic tool validation failures

## Compaction Ownership

Compaction is a runtime policy, not a loop behavior.

## Compaction trigger candidates

- provider context overflow
- proactive token budget threshold
- session size threshold

## Compaction invariants

- full original history remains recoverable
- summary artifact carries `RuntimeSummary` semantics
- recent exact turns stay verbatim
- compaction record links source message range to produced summary artifact

## Suggested compaction flow

1. `session-runtime` freezes current run state
2. selects old message window for compaction
3. invokes summarization through a controlled subroutine
4. persists a compaction record
5. rebuilds active context with summary plus recent exact messages
6. starts a continuation run

## Cancellation Model

Cancellation must be explicit because MatrixClaw will serve multiple surfaces.

Sources of cancellation:

- CLI interrupt
- web UI stop button
- compatibility client disconnect with cancel intent
- server shutdown

Required behavior:

- stop provider stream if possible
- stop or signal running tools if policy permits
- persist a terminal cancellation marker if the user saw one
- leave the session in a resumable state

## Concurrency Model

Initial runtime recommendation:

- per-session single active run
- multi-session concurrency allowed
- tool execution may use bounded internal concurrency

Reasons:

- avoids ordering ambiguity
- simplifies persistence
- makes protocol bridging safer

## Persistence Boundaries

The loop remains pure only if persistence boundaries are clear.

## Must persist immediately

- inbound user messages
- finalized assistant messages
- tool call records
- tool result messages
- visible terminal warnings and errors
- run metadata needed for resume

## May persist asynchronously

- fine-grained stream deltas
- debug metrics
- verbose execution logs

## Resume Semantics

After restart, `session-runtime` should reconstruct:

- durable transcript
- queue state
- last provider/model choice
- pending retry metadata if any

It should not attempt to resume an interrupted in-flight provider stream byte-for-byte.
Instead it should:

- mark the previous run incomplete or cancelled
- begin a new continuation run if operator policy requests it

## Design Checks

The runtime model is correct if all of these stay true:

- one assistant generation produces one durable final answer
- the transcript always matches what the user saw
- queue insertion points are deterministic
- retry and compaction are policies above the loop
- no compatibility surface forces internal message types to mirror external payloads
