# MatrixClaw Runtime Architecture

## Overview

MatrixClaw is a single-binary Rust agent runtime. No Node.js, no Electron, no Tauri.
One `matrixclaw` binary provides a TUI chat interface, an HTTP/SSE API, and an MCP client.

## Crate Structure

```
matrixclaw-app-host       CLI entry point, HTTP server, chat REPL
├── matrixclaw-provider   Provider control plane (registry, fallback, cost, rate limit, health)
│   └── matrixclaw-agent-core  Async ReAct loop, Provider trait, policy engine
├── matrixclaw-tools       Tool registry, 13 built-in tools, MCP client
├── matrixclaw-session-runtime  SQLite storage, queue, compaction, recovery
├── matrixclaw-manifests   Plugin and skill manifest types
└── matrixclaw-compat-openclaw  OpenClaw gateway adapter
```

## Agent Loop

The agent core runs an async ReAct loop (`crates/agent-core/src/loop.rs`):

1. Send context + tool descriptors to provider via `Provider::stream()`
2. Receive streaming response with structured tool calls (JSON function-calling)
3. For each tool call: emit events, execute via `ToolRegistry`, append result to context
4. Repeat until the provider returns a text-only response (no tool calls) or max iterations reached

The loop emits `AgentEvent` variants for observability: `RunStarted`, `MessageStarted`,
`MessageDelta`, `MessageCompleted`, `ToolCallReceived`, `ToolExecutionStarted`,
`ToolExecutionCompleted`, `RunCompleted`.

A `ToolPreflightPolicy` can intercept and block tool calls before execution.

## Tool System

### Registry

`ToolRegistry` holds `Arc<dyn ToolExecutor>` entries behind `RwLock<HashMap>`.
Tools are registered by name and discovered by the agent loop via `ToolDescriptor::to_openai_function()`.

### Built-in Tools (13)

| Tool | Status | Description |
|------|--------|-------------|
| terminal | full | Shell command execution with timeout |
| read_file | full | File reading with line numbers, offset/limit |
| write_file | full | File creation with auto-mkdir |
| list_directory | full | Directory listing with optional recursion |
| edit_file | full | Find/replace file editing |
| web_fetch | full | HTTP requests via reqwest |
| web_search | stub | Requires search provider API key |
| calculator | full | Expression evaluator (+, -, *, /, parens) |
| environment | full | Env vars and system info |
| memory | full | In-memory key-value store |
| code_interpreter | stub | Phase 5 |
| delegate | stub | Phase 3 |
| skills | stub | Phase 4 |

### MCP Client

External tools are loaded via the MCP protocol (JSON-RPC over stdio).
MCP servers are configured in `~/.matrixclaw/config/mcp.json`.
Tools are namespaced as `mcp__{server}__{name}` and wrapped as `ToolExecutor` implementations.

## Provider Layer

The `Provider` trait (`crates/agent-core/src/provider.rs`) defines async `complete()` and `stream()` methods.
`FallbackProvider` (`crates/provider-plane/src/fallback.rs`) wraps the registry and implements `Provider` —
the agent loop sees a single provider transparently.

`OpenAiProvider` (`crates/provider-plane/src/openai.rs`) handles:

- Chat completion with function-calling (`tools` / `tool_choice` fields)
- SSE streaming with incremental tool call argument assembly
- Token usage extraction from response `usage` fields
- Works with any OpenAI-compatible endpoint (OpenRouter, Ollama `/v1`, etc.)

## Provider Plane

The provider control plane (`crates/provider-plane/`) sits between `agent-core` and `app-host`.

### Registry

`ProviderRegistry` maps named provider configs to `Box<dyn Provider>` instances.
Providers are configured via `~/.matrixclaw/config/providers.json` or built from env vars.

### Fallback Chains

`FallbackProvider` implements `Provider` and tries providers in chain order.
Skips unhealthy providers (tracked by `HealthChecker`) and rate-limited providers (tracked by `RateLimiter`).
On failure, marks provider unhealthy and tries next in chain.

### Token Counting

`TokenUsage` is extracted from OpenAI `usage` response fields. Accumulated in-memory per session/model.

### Cost Tracking

`CostTracker` persists cost records to SQLite. Queryable by session or model.

### Rate Limiting

`RateLimiter` uses an atomic token-bucket per provider (configurable RPM).

### Health Checks

`HealthChecker` tracks healthy/unhealthy state per provider. Supports async HTTP probes.

## Session Runtime

Sessions are persisted in SQLite (`crates/session-runtime/src/sqlite.rs`).
Each session stores the full message history, supports compaction for long conversations,
and can be recovered after crashes.

## CLI

```
matrixclaw                Launch TUI chat REPL
matrixclaw chat           Same as above (explicit subcommand)
matrixclaw llm-smoke      Run a single LLM round-trip (requires API key)
```

Chat mode supports:
- Streaming output with readline prompt
- Tool call/result display in terminal
- Session persistence across turns
- `/model` in-chat command and `--model` flag
- `MATRIXCLAW_LLM_MODEL` env var for default model

## Configuration

```
~/.matrixclaw/config/
├── config.json       General settings
├── mcp.json          MCP server definitions
└── providers.json    Provider configs and fallback chains
```

## Event Flow

```
User prompt
  → Agent loop (run_prompt / run_prompt_with_policy)
    → Provider::stream() → SSE chunks → AgentEvent::MessageDelta
    → Tool calls parsed from JSON → AgentEvent::ToolCallReceived
    → Policy check (optional) → allow or block
    → ToolRegistry::execute() → AgentEvent::ToolExecutionCompleted
    → Results appended to context
    → Loop back to provider
  → Final text response → AgentEvent::RunCompleted
```
