# MatrixClaw Runtime Architecture

## Overview

MatrixClaw is a single-binary Rust agent runtime. No Node.js, no Electron, no Tauri.
One `matrixclaw` binary provides a TUI chat interface, an HTTP/SSE API, an MCP client, and an MCP server.

## Crate Structure

```
matrixclaw-app-host       CLI entry point, HTTP server, chat REPL
├── matrixclaw-provider   Provider control plane (registry, fallback, cost, rate limit, health)
│   └── matrixclaw-agent-core  Async ReAct loop, Provider trait, policy engine
├── matrixclaw-tools       Tool registry, 18 built-in tools, MCP client
├── sandwrench             Sandbox abstraction — Docker, E2B, Daytona, Local backends
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
`ToolExecutionCompleted`, `IterationPressure`, `RunCompleted`.

A `ToolPreflightPolicy` can intercept and block tool calls before execution.
`CompositeHook` dispatches lifecycle events to registered `LifecycleHook` implementations,
enabling pre/post interception of LLM calls, tool calls, and session events.

## Tool System

### Registry

`ToolRegistry` holds `Arc<dyn ToolExecutor>` entries behind `RwLock<HashMap>`.
Tools are registered by name and discovered by the agent loop via `ToolDescriptor::to_openai_function()`.

### Built-in Tools (20 core + 9 browser)

| Tool | Status | Description |
|------|--------|-------------|
| terminal | full | Shell command execution with timeout |
| read_file | full | File reading with line numbers, offset/limit |
| write_file | full | File creation with auto-mkdir |
| list_directory | full | Directory listing with optional recursion |
| edit_file | full | Find/replace file editing |
| patch | full | Fuzzy file editing with 6 strategies (exact, prefix, suffix, contains, fuzzy, regex) |
| web_fetch | full | HTTP requests via reqwest |
| web_search | full | SearXNG-backed web search |
| calculator | full | Expression evaluator (+, -, *, /, parens) |
| environment | full | Env vars and system info |
| memory | full | Persistent key-value store with search, survives restarts (SQLite) |
| code_interpreter | full (sandwrench-backed) | Sandboxed code execution via sandwrench abstraction |
| delegate | full | Subagent spawning with callback-based architecture |
| delegate_parallel | full | Parallel subagent execution via tokio::spawn |
| skills | full | List, read, and create skills in ~/.matrixclaw/skills/ |
| search_files | full | Ripgrep-backed content search with path traversal protection |
| todo | full | Session-scoped task list for multi-step work |
| clarify | full | Structured user questions with optional multiple-choice |
| process | full | Background process management (list/register/kill) |
| session_search | full | FTS5 full-text search across conversation history |
| cronjob | full | Scheduled task execution with SQLite-backed job store |
| browser_navigate | full (feature-flagged) | Navigate to URL |
| browser_snapshot | full (feature-flagged) | Page content snapshot |
| browser_click | full (feature-flagged) | Click element by CSS selector |
| browser_type | full (feature-flagged) | Type text into element |
| browser_scroll | full (feature-flagged) | Scroll page up/down |
| browser_go_back | full (feature-flagged) | Navigate back in history |
| browser_get_url | full (feature-flagged) | Get current page URL |
| browser_screenshot | full (feature-flagged) | Capture PNG screenshot |
| browser_close | full (feature-flagged) | Close browser and release resources |

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
- Prompt caching hints for Anthropic/Gemini models via OpenRouter `cache_control` in system prompt

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

## Multi-Agent Orchestration

The `delegate` tool (`crates/matrixclaw-tools/src/builtin/delegate.rs`) enables an agent to spawn child agents.
The `delegate_parallel` tool (`crates/matrixclaw-tools/src/builtin/delegate_parallel.rs`) runs multiple child agents concurrently.

### Architecture

Uses a callback pattern to avoid circular dependencies between `matrixclaw-tools` and `agent-core`:

- `SubagentRunner` — callback for single subagent execution
- `ParallelSubagentRunner` — callback for parallel subagent execution
- `DelegateTool` holds a `SubagentRunner` callback and a depth counter
- `DelegateParallelTool` holds a `ParallelSubagentRunner` callback
- Callbacks are created in `app-host` where both the provider and registry are available

### Parallel Execution

Each parallel subagent gets its own `FallbackProvider` instance created from the shared `Arc<ProviderRegistry>`.
This avoids Mutex contention — subagents make LLM calls concurrently via `tokio::spawn`.

- Shared: `Arc<ProviderRegistry>` (cheap clone), `Arc<ToolRegistry>` (thread-safe)
- Per-subagent: `FallbackProvider` (own rate limiter state)
- Uses `tokio::spawn` for true parallelism across the tokio thread pool
- `chat.rs` wraps the `FallbackProvider` in `Arc<tokio::sync::Mutex>` for safe sharing between the main loop and subagent runner

### Depth Limiting

Max depth 2. At max depth, `delegate` returns an error instead of spawning. The depth counter increments for each nesting level.

### Wiring

 `SessionBackedLiveRunService::register_delegate_tool()` registers a `DelegateTool` and `DelegateParallelTool` into the tool registry. Called once during service setup in `chat.rs` and `delegate_parallel` section after the `delegate` section:

## Iteration Budget

The agent loop runs a maximum of 90 iterations (configurable via `RunRequest::max_iterations`).
Pressure warnings are emitted as `AgentEvent::IterationPressure` at 70% (iteration 63) and 90% (iteration 81).
The `live_runtime` surfaces these as `"iteration_pressure"` events.

## Session Runtime

Sessions are persisted in SQLite (`crates/session-runtime/src/sqlite.rs`).
Each session stores the full message history, supports compaction for long conversations,
and can be recovered after crashes.

FTS5 virtual tables enable full-text search across all session message history.
Context compression (`crates/session-runtime/src/compression.rs`) implements a 4-phase
Hermes-style strategy: prune tool results → identify turn boundaries → summarize old turns →
reassemble compressed context.

## Lifecycle Hooks

The hook system (`crates/agent-core/src/hooks.rs`) provides extensible interception points
throughout the agent loop. Hooks are dispatched from `run_prompt_with_policy` in `loop.rs`.

### Types

- **`HookPoint`** — enum with six variants: `PreLlmCall`, `PostLlmCall`, `PreToolCall`,
  `PostToolCall`, `OnSessionStart`, `OnSessionEnd`
- **`HookPayload`** — structured event payload carrying `hook_point`, `session_id`,
  `tool_name`, `tool_arguments`, `tool_result`, `llm_response`, and `iteration`
- **`HookAction`** — return type with `block: bool` and optional `reason`. `HookAction::allow()`
  permits continuation; `HookAction::block(reason)` halts execution with a message
- **`LifecycleHook`** — async trait (`on_event`, `name`) that hook implementations satisfy
- **`CompositeHook`** — holds `Vec<Box<dyn LifecycleHook>>`, dispatches events in order,
  short-circuits on first block

### Dispatch Points in the Agent Loop

1. **Pre-LLM call** — before `Provider::stream()`. A blocking hook terminates the run.
2. **Post-LLM call** — after receiving provider response. Observation only.
3. **Pre-tool call** — before `ToolRegistry::execute()`. A blocking hook returns a hooked
   result to the provider instead of executing the tool.
4. **Post-tool call** — after tool execution completes. Observation only.
5. **On session start/end** — dispatched by session lifecycle management.

## Command Approval

`ApprovalChecker` (`crates/agent-core/src/approval.rs`) inspects terminal commands
before execution. Regex patterns detect dangerous operations (rm -rf, sudo, chmod 777, etc.).
Approval policies (AllowAll, DenyDangerous, RequireApproval) are configurable per session.

## Cron Scheduling

`CronjobTool` (`crates/matrixclaw-tools/src/builtin/cronjob.rs`) enables agents to schedule
recurring tasks. Jobs are stored in SQLite with cron expressions. The store supports add, remove,
list, and tick (execute due jobs). Integrated with the agent's tool system so scheduled jobs
run as agent prompts.

## MCP Server Mode

`matrixclaw mcp-serve` starts an MCP server (JSON-RPC over stdio) that exposes MatrixClaw's
tools to external clients (IDEs, other agents, scripts). Implements the MCP protocol's
`tools/list` and `tools/call` methods.

## Sandbox Backends

### Sandwrench (`sandwrench` crate)

`sandwrench` provides a unified sandbox abstraction layer for isolated code and command execution.
It decouples the `code_interpreter` tool (and future tools) from any single backend implementation.

#### Core Trait

`SandboxRuntime` (`crates/sandwrench/src/lib.rs`) defines the sandbox interface:

- `execute_code(language, code, working_dir) -> SandboxResult` — run code in an isolated environment
- `execute_command(cmd, args, working_dir, env) -> SandboxResult` — run a shell command in the sandbox

#### Backends

| Backend | Type | Description |
|---------|------|-------------|
| Docker | container (default) | Local Docker containers with resource limits and automatic cleanup |
| E2B | cloud microVM | E2B cloud sandbox for on-demand ephemeral VMs |
| Daytona | self-hosted | Daytona workspace API for self-hosted sandbox environments |
| Local | passthrough | Direct execution on the host (no isolation, for development) |

#### Factory

`SandboxProvider` reads backend selection from config and constructs the appropriate
`SandboxRuntime` implementation. The `code_interpreter` tool requests a sandbox via the provider
rather than directly depending on any backend.

#### Configuration

Sandbox backend is configured via `~/.matrixclaw/config/sandbox.json`:

```json
{
  "backend": "docker",
  "docker": { "image": "matrixclaw-sandbox:latest", "memory_mb": 512, "timeout_secs": 120 },
  "e2b": { "api_key_env": "E2B_API_KEY", "template_id": "base" },
  "daytona": { "server_url": "http://localhost:3986", "api_key_env": "DAYTONA_API_KEY" }
}
```

### DockerSandbox

`DockerSandbox` (`crates/matrixclaw-tools/src/sandbox.rs`) is the original Docker backend.
It is now wrapped by `sandwrench`'s Docker backend implementation, which delegates to the same
underlying Docker API while conforming to the `SandboxRuntime` trait.

## CLI

```
matrixclaw                Launch TUI chat REPL
matrixclaw chat           Same as above (explicit subcommand)
matrixclaw llm-smoke      Run a single LLM round-trip (requires API key)
matrixclaw mcp-serve      Start MCP server (JSON-RPC over stdio)
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
├── providers.json    Provider configs and fallback chains
└── sandbox.json      Sandbox backend selection and backend-specific config
```

## Event Flow

```
User prompt
  → Agent loop (run_prompt / run_prompt_with_policy)
    → LifecycleHook dispatch (PreLlmCall) — optional, can block
    → Provider::stream() → SSE chunks → AgentEvent::MessageDelta
    → LifecycleHook dispatch (PostLlmCall) — observation only
    → Tool calls parsed from JSON → AgentEvent::ToolCallReceived
    → LifecycleHook dispatch (PreToolCall) — optional, can block
    → Policy check (optional) → allow or block
    → ToolRegistry::execute() → AgentEvent::ToolExecutionCompleted
    → LifecycleHook dispatch (PostToolCall) — observation only
    → Results appended to context
    → Loop back to provider
  → Final text response → AgentEvent::RunCompleted
```

## Model Routing

Config-driven routing selects the best provider+model for each task based on prompt characteristics.

1. **RoutingRule** — defines a named route with match criteria (skills, keywords, max_prompt_chars, tool_count_min) and target provider+model
2. **ModelRouter** — evaluates rules in order, first match wins, falls back to the default provider chain
3. **Config** — routes live in `providers.json` alongside provider definitions, under the `routes` key

Example: short prompts go to a fast local model, skill-heavy prompts go to a capable cloud model, complex multi-tool tasks get routed to the most capable provider.

## Self-Evolving Skills

Skills automatically improve from execution feedback through three components:

1. **TraceCollector** (LifecycleHook) — observes every `PostToolCall` event, buffers tool invocations for the active skill, and records complete execution traces (success/failure/partial) into `~/.matrixclaw/state/skill_traces.sqlite3`
2. **TraceAnalyzer** — groups traces by skill, computes success rates, detects repeated failure patterns in tool-chain sequences via sliding-window analysis, and gathers success/failure example summaries
3. **SkillRewriter** — when a skill's success rate drops below 50% with 3+ traces, constructs a rewrite prompt containing the current skill instructions + failure patterns + examples, calls the LLM to generate improved instructions, writes as new version with automatic archival of the previous version

**Key design decisions:**
- No Python dependency — pure Rust implementation inspired by DSPy's GePA and MiProv2
- `matrixclaw-hooks` crate extracted to break circular dependency between `matrixclaw-tools` and `agent-core`
- Callback pattern (`LlmRewriteFn`) for LLM calls avoids direct provider dependency in tools crate
- Skill versioning archives previous versions to `v<N>.md` before rewriting, enabling rollback
- Triggered automatically via lifecycle hooks or manually via `skill_evolve` tool
