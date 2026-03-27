# Architecture

## Architectural Principles

### 1. Core loop purity

The agent loop must not own:

- session persistence
- install/setup logic
- compatibility protocol handling
- retry policy
- compaction policy
- product-specific UI concerns

It should own only:

- turn orchestration
- streaming assistant lifecycle
- tool preflight, execution, and result propagation
- queue-aware continuation rules
- structured event emission

### 2. Streaming-first

Streaming is the canonical execution path. Non-streaming consumers derive their output from the same event stream or final message object, never from a separate second completion.

### 3. Boundary isolation

The internal runtime should speak internal Rust types. OpenClaw-compatible WebSocket and HTTP payloads must be translated at the compatibility boundary.

### 4. Binary-first operations

The product should work as a native binary out of the box. Extra assets may exist, but they are lazy-managed components, not mandatory setup prerequisites.

## System Layers

## Layer 1: `agent-core`

Purpose: generic loop engine reusable across CLI, TUI, web, RPC, and compatibility APIs.

### Responsibilities

- maintain in-memory agent context for the current run
- accept prompt or continue requests
- stream assistant output events
- detect and execute tool calls
- emit tool execution lifecycle events
- process steering and follow-up queues

### Core Rust types

- `AgentMessage`
  - rich internal message enum, not limited to provider wire roles
- `LlmMessage`
  - provider-facing message shape derived from internal messages
- `AgentEvent`
  - `agent_start`, `turn_start`, `message_start`, `message_delta`, `message_end`, `tool_execution_start`, `tool_execution_update`, `tool_execution_end`, `turn_end`, `agent_end`
- `AgentContext`
  - system prompt, active messages, active tools, provider/model handle
- `ToolCall`, `ToolResult`
- `RunMode`
  - prompt vs continue

### Core traits

- `Provider`
  - streaming LLM interface
- `Tool`
  - validated executable tool contract
- `ContextTransformer`
  - internal-message to internal-message transforms before provider conversion
- `LlmMessageConverter`
  - internal-message to provider-message conversion
- `ToolPolicy`
  - optional `before_tool_call` and `after_tool_call` hooks

## Layer 2: `session-runtime`

Purpose: durable runtime policy and session semantics above the loop.

### Responsibilities

- persistent session storage
- queue storage for steering/follow-up messages
- compaction strategy
- overflow recovery and retry strategy
- workspace memory file integration
- session switching, forking, and branching
- event consumption and persistence

### Key rule

`session-runtime` consumes `AgentEvent`s from `agent-core` and updates durable state. The core loop itself does not write to disk.

### Storage model

Initial v1:

- SQLite for indexed runtime state
- append-only JSONL export or snapshot support for portability

Suggested persisted entities:

- sessions
- session_messages
- queued_messages
- tool_executions
- compaction_records
- compatibility_tokens or API tokens
- managed_assets
- agent_workspaces metadata

## Layer 3: `compat-openclaw`

Purpose: expose OpenClaw-facing APIs without distorting the internal runtime.

### Responsibilities

- WebSocket handshake and request/response framing
- OpenAI-compatible HTTP endpoints where useful
- OpenClaw header handling
- session import/export compatibility helpers
- capability negotiation and versioning
- skill/package compatibility classification and import helpers
- plugin adaptation for supported external plugin boundaries

### Rules

- no internal logic is allowed to depend on OpenClaw frame types
- protocol requests are translated into internal `session-runtime` operations
- protocol responses are rendered from internal events and state
- ecosystem compatibility must prefer translation or process-boundary adapters over in-process emulation of JS runtime internals

## Compatibility Subsystems

### `compat-openclaw-protocol`

Purpose:

- client and API interoperability

Responsibilities:

- WebSocket compatibility frames
- HTTP compatibility routes
- auth/header translation
- session import/export adapters

### `compat-openclaw-ecosystem`

Purpose:

- skill and plugin compatibility

Responsibilities:

- import OpenClaw-style text skills and prompt bundles
- support workspace convention compatibility
- classify plugin artifacts by compatibility tier
- adapt subprocess, JSON-RPC, and MCP-style plugins
- reject or gate in-process JS/Bun-only extensions with explicit diagnostics

## Layer 4: `app-host`

Purpose: product shell and operator experience.

### Responsibilities

- CLI entrypoint
- daemon/service mode
- first-run setup server or TUI wizard
- embedded web assets
- channel bootstrapping
- managed asset download service
- configuration loading and migration

## Execution Flow

## Prompt flow

1. `app-host` receives a prompt from CLI, web UI, or compatibility API
2. `session-runtime` resolves session and appends the inbound user/app message
3. `session-runtime` creates an `agent-core` run request
4. `agent-core` emits stream events while updating in-memory context
5. `session-runtime` persists those events and derived messages
6. `app-host` and adapters forward stream updates to clients
7. run ends when there are no more tool calls, steering messages, or follow-ups

## Continue flow

1. `session-runtime` decides a continue is needed
2. `agent-core` resumes from existing in-memory or reconstructed context
3. no synthetic user message is appended unless policy requires it

## Overflow recovery flow

1. provider returns overflow or a classified retryable failure
2. `session-runtime` classifies the error
3. if overflow:
  - remove transient failure artifact from active retry context if needed
  - compact messages
  - append compaction metadata
  - invoke `continue`
4. if transient retryable:
  - schedule bounded backoff retry
  - invoke `continue`

## Event Model

Events are the integration contract across layers.

### Required properties

- deterministic ordering
- serializable for persistence and replay
- sufficient for UI rendering without inspecting hidden mutable state
- sufficient for compatibility adapters to generate external protocol frames

### Important event semantics

- assistant `message_end` is the barrier before tool preflight
- tool result messages are first-class messages, not out-of-band strings
- every terminal user-visible message must be emitted and persisted
- warning/failure events that affect conversation meaning should have message equivalents when exposed to the user

## Tool Execution Design

### Preflight phase

- validate tool existence
- validate arguments against schema
- run `before_tool_call` policy/hook
- block with a structured result if disallowed

### Execution phase

- support sequential and parallel execution modes
- support progress updates where available
- preserve assistant source order for final tool result insertion

### Post-processing phase

- run `after_tool_call`
- normalize errors into structured result objects
- emit `tool_execution_end`
- emit and persist `toolResult` message

## Compaction Design

Compaction is not part of the core loop. It is a runtime policy.

### Requirements

- preserve full pre-compaction history in durable storage
- produce a summary artifact with explicit non-user role semantics
- keep recent exact messages and compress older context
- run under cancellation-aware contexts

### Recommended summary insertion

Use an internal summary message role such as:

- `runtime_summary`
- or a system-style summary artifact converted appropriately by the LLM converter

Do not rewrite compaction summaries as user-authored messages.

## Workspace and Memory Model

Each agent workspace should keep human-editable files such as:

- `SOUL.md`
- `IDENTITY.md`
- `AGENTS.md`
- `USER.md`
- `MEMORY.md`
- `HEARTBEAT.md`
- `TOOLS.md`

The runtime may also maintain:

- searchable conversation logs
- compaction snapshots
- managed plugin/tool state

These files are session-runtime inputs, not core-loop responsibilities.

## Skill and Plugin Compatibility Design

### Skills

Skills should be treated as data-first artifacts.

Recommended v1 behavior:

- support markdown/text skill files directly
- support OpenClaw-style prompt and workspace file conventions
- normalize imported skill metadata into native MatrixClaw records
- preserve original source location for provenance and update tracking

### Plugins

Plugins should be divided by execution boundary.

#### Native-compatible plugins

- subprocess plugins
- JSON-RPC plugins
- MCP servers

These can be adapted into native Rust tool/channel/provider abstractions.

#### Bridge-only plugins

- Node/Bun plugins that communicate only through a stable process protocol

These may be supported later via an optional bridge runtime.

#### Unsupported as native targets

- in-process TypeScript extensions tightly coupled to OpenClaw or `pi` internals
- plugins that assume direct access to internal JS objects and lifecycle hooks

These should fail with a clear compatibility diagnostic rather than silently degrading.

## Binary Distribution and Managed Assets

### Install path

Default installer target:

- `~/.matrixclaw/bin/matrixclaw`

This avoids forced privileged writes and aligns with the “no extra setup steps” goal.

### Embedded assets

Embed:

- setup UI static files
- core web UI static files
- default templates and workspace bootstrap files

### Managed optional assets

Lazy-install on first use:

- browser engine
- OCR/STT models
- heavyweight viewer helpers

Track these in a managed asset registry with versioning and checksum verification.

## Suggested Rust workspace layout

```text
crates/
  agent-core/
  session-runtime/
  compat-openclaw/
  compat-openai/
  app-host/
  web-ui-assets/
  setup-ui-assets/
  tools/
  providers/
  storage/
```

## Recommended v1 implementation order

1. `agent-core`
2. `session-runtime`
3. `compat-openclaw`
4. `app-host` with embedded setup/web UI
5. optional channels and managed assets

## Architecture Decisions

### Decision: streaming-first assistant generation

Chosen because it avoids duplicated generations, keeps UI and persistence aligned, and matches the strongest part of `pi-agent-core`.

### Decision: compatibility as adapter, not architecture

Chosen because it protects internal coherence while still enabling OpenClaw integration.

### Decision: ecosystem compatibility is tiered

Chosen because “supports OpenClaw plugins” is too broad to be honest unless support is split into native, shimmed, and unsupported classes.

### Decision: SQLite-first persistence

Chosen because it fits local self-hosting, supports rich indexing, and still allows import/export compatibility tooling.

### Decision: no mandatory Docker dependency

Chosen because the product goal is native binary installation. Sandboxing may exist, but must be optional and explicit.
