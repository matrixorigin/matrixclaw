# MatrixClaw Runtime Architecture

## Overview

MatrixClaw is a single-binary Rust agent runtime. No Node.js, no Electron, no Tauri.
One `matrixclaw` binary provides a TUI chat interface, an HTTP/SSE API, and an MCP client.

## Crate Structure

```
matrixclaw-app-host       CLI entry point, HTTP server, chat REPL, OpenAI provider
├── matrixclaw-agent-core  Async ReAct loop, Provider trait, policy engine
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
The OpenAI-compatible provider (`crates/app-host/src/openai_compatible.rs`) handles:

- Chat completion with function-calling (`tools` / `tool_choice` fields)
- SSE streaming with incremental tool call argument assembly
- Works with any OpenAI-compatible endpoint (OpenRouter, etc.)

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
├── config.toml        General settings
└── mcp.json           MCP server definitions
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
