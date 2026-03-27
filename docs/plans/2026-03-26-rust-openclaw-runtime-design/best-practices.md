# Best Practices

## Architectural Best Practices

### Learn the loop from `pi-mono`

Adopt these patterns directly:

- use rich internal agent messages and convert to provider messages only at the provider boundary
- make streaming the primary execution path
- emit structured events for every meaningful state transition
- support queued steering and follow-up messages in the core loop contract
- separate tool preflight, execution, and post-processing
- keep retry, compaction, persistence, and UI concerns above the core loop

### Learn packaging from FastClaw

Adopt these product instincts:

- single-binary operator experience
- embedded first-run setup
- embedded web UI assets
- self-hosted default runtime
- compatibility-facing HTTP/WebSocket endpoints

### Do not inherit FastClaw's current loop weaknesses

Avoid:

- double-generation for final streamed answers
- terminal fallback messages that are not persisted
- compaction summaries written back as user messages
- product/runtime logic being tightly fused to the loop
- shipping a compatibility claim without protocol fixture tests

## Loop Semantics Best Practices

### Exactly-once final answer generation

The same assistant generation must satisfy:

- streaming output
- final persisted assistant message
- compatibility/API output

Never generate once for control flow inspection and a second time for user-visible streaming.

### Barrier semantics before tool execution

The assistant message that requested tool calls must be finalized in runtime state before tool preflight begins. Policies and hooks should see a consistent context snapshot.

### Tool results as first-class messages

Tool outputs should be modeled as structured result messages, not appended as raw strings outside session semantics.

### Structured terminal failures

If the user sees a warning, refusal, or “max iterations reached” message, that message must become part of durable session history.

## Persistence Best Practices

### Event-sourced thinking, message-based durability

Persist:

- a durable message log
- selected structured execution metadata
- enough event detail for replay and debugging

Do not rely solely on ephemeral in-memory state or UI reconstruction.

### Preserve pre-compaction history

Compaction should reduce active context, not destroy recoverable history.

Recommended:

- keep full message history in SQLite
- optionally write JSONL exports or snapshots
- record compaction summaries and provenance

## Compatibility Best Practices

### Separate protocol compatibility from ecosystem compatibility

Do not use “OpenClaw compatibility” as a single catch-all label.

Track separately:

- client/protocol compatibility
- ecosystem compatibility for skills and plugins

### Use fixtures, not assumptions

For OpenClaw compatibility:

- collect real client request/response fixtures
- test handshake and framing explicitly
- version compatibility claims by capability

Avoid vague “compatible” claims without a tested surface matrix.

### Compat boundary only

External payloads should be translated once at the edge. Internal code should operate on native domain types.

### Support skills as data, not code hooks

Prefer strong compatibility for:

- markdown/text skills
- prompt bundles
- workspace convention files

These are stable and portable.

### Tier plugin compatibility honestly

Document plugin support explicitly:

- native
- shimmed
- unsupported

Do not imply that all OpenClaw plugins are portable just because some subprocess plugins are.

### Prefer process boundaries for third-party extensibility

When supporting external ecosystems, prefer:

- MCP
- JSON-RPC subprocesses
- stable CLI bridges

Avoid coupling MatrixClaw core architecture to foreign in-process extension APIs.

## Installation Best Practices

### User-owned install paths by default

Prefer:

- `~/.matrixclaw/bin`
- shell profile hints or wrapper symlinks in user-owned directories

Avoid:

- default `/usr/local/bin` writes
- mandatory `sudo`
- installers that implicitly depend on package managers being present

### Managed lazy downloads

Heavy components should:

- download on first use
- verify checksum/signature
- record installed version
- be removable and re-fetchable

## Security Best Practices

### Optional sandboxing, mandatory explicitness

The runtime should work without Docker or other sandbox engines. If sandboxing is enabled:

- declare which backend is active
- show clear failure messages if the backend is unavailable
- expose policy controls explicitly

### Hook and plugin trust boundaries

Treat:

- local tools
- plugins
- MCP servers
- compatibility clients

as separate trust zones. Each should have distinct validation and policy hooks.

## Performance Best Practices

### Parallel tools with ordered emission

Allow parallel execution where safe, but preserve deterministic assistant-order result emission to keep sessions reproducible.

### Avoid expensive system prompt rebuilding per tiny event

System prompt and workspace-derived context should be cached and invalidated deliberately, not rebuilt for every superficial state update.

## Testing Best Practices

### Test layers independently

`agent-core`

- stream lifecycle
- tool execution ordering
- queued steering/follow-up semantics
- cancellation

`session-runtime`

- persistence parity
- compaction correctness
- overflow recovery
- retry behavior

`compat-openclaw`

- handshake fixtures
- request translation
- response framing

`app-host`

- first-run setup
- embedded asset serving
- managed asset installs

### Golden transcript tests

Maintain golden transcripts that assert:

- streamed deltas
- final messages
- persisted session records
- compatibility-facing outputs

all describe the same run.

## Recommended Decision Rules

- If a behavior belongs to every future MatrixClaw surface, it belongs in `agent-core`.
- If a behavior depends on persistence, policy, or operator preference, it belongs in `session-runtime`.
- If a behavior exists only to satisfy an external ecosystem contract, it belongs in a compatibility crate.
- If a feature increases install complexity, make it lazy, optional, or separate from first-run success.
