# MatrixClaw Runtime Rethink — 8-Phase Roadmap

**Date**: 2026-03-31
**Status**: Phase 8 In Progress — SSH sandbox, agent lifecycle, self-nudging, messaging gateway done; model routing, profiles, DSPy remain
**Decision**: Drop SvelteKit/Tauri desktop shell. Rebuild as single-binary Rust agent runtime.

## Differentiators

1. **Single-binary simplicity** — one `matrixclaw` binary, no Node.js, no Electron
2. **Provider/cost control plane** — multi-provider routing with fallback chains, cost tracking, prompt caching
3. **Multi-agent orchestration** — per-agent profiles with capability bindings, subagent delegation

## What We Keep

- `session-runtime` — sessions, queue, compaction, SQLite storage, recovery
- `agent-core` loop — ReAct pattern, streaming (being updated for async + JSON function-calling)
- Agent profiles, session bindings, manifests
- Gateway adapter trait, OpenClaw compat layer
- Config system (`~/.matrixclaw/config/`)

## What We Drop

- `ui/` — entire SvelteKit frontend (deleted)
- Figma design system work (deprecated)
- `DESIGN.md` as UI doc (rewrite as runtime arch doc)
- Tauri desktop shell integration

## What We Build

---

## Phase 1: WASM Tool Engine → Native Tool System

**Goal**: Replace 2 hardcoded tools with a proper async tool registry, 13 built-in tools, and JSON function-calling.

**Status**: Phase 1 Complete

### Stage 1.1 — Core Types (matrixclaw-tools crate)
- [x] `ToolDescriptor` with `to_openai_function()` conversion
- [x] `ToolParameter` with JSON Schema types
- [x] Async `ToolExecutor` trait (`&self`, async execute)
- [x] `ToolRegistry` with `Arc<RwLock<HashMap>>` for concurrent access
- [x] `ToolCall` and `ToolResult` with id-based correlation

### Stage 1.2 — 13 Built-in Tools
- [x] terminal — shell command execution with timeout
- [x] read_file — file reading with line numbers, offset/limit
- [x] write_file — file creation with auto-mkdir
- [x] list_directory — directory listing with optional recursion
- [x] edit_file — find/replace file editing
- [x] web_fetch — HTTP requests via reqwest
- [x] web_search — (stub, needs search API key)
- [x] calculator — expression evaluator (+, -, *, /, parens)
- [x] environment — env vars, system info
- [x] memory — in-memory key-value store
- [x] code_interpreter — (stub, Phase 5)
- [x] delegate — (stub, Phase 3)
- [x] skills — (stub, Phase 4)

### Stage 1.3 — Agent Core Update
- [x] Switch from `call:name(args)` text parsing to JSON function-calling
- [x] Make `Provider` trait async
- [x] Make agent loop async
- [x] Add `ToolCallDelta` streaming event
- [x] Update `RunRequest` with `tools` and `tool_choice` fields

### Stage 1.4 — App Host Wiring
- [x] Replace `AppToolExecutor` with `ToolRegistry`
- [x] Update OpenAI provider for function-calling API
- [x] Remove hardcoded `add` and `host.command` dispatch

### Stage 1.5 — MCP Client (future plugin path)
- [x] MCP client protocol (JSON-RPC over stdio)
- [x] MCP tool adapter wrapping `ToolExecutor`
- [x] Dynamic tool discovery from MCP servers

### Stage 1.6 — Cleanup
- [x] Delete `ui/` directory
- [x] Remove `[package.metadata.matrixclaw.ui]` from app-host
- [x] Rewrite DESIGN.md as runtime architecture doc
- [x] Update CLAUDE.md

### Stage 1.7 — TUI Chat Mode (USER PRIORITY)
- [x] Add `matrixclaw chat` subcommand (or make bare `matrixclaw` launch chat)
- [x] Readline-based REPL with streaming output
- [x] Session persistence across chat turns
- [x] Tool call/result display in terminal
- [x] Model selection via `--model` flag or config

**Tool Plugin Strategy**: WASM deferred in favor of MCP protocol for third-party tools. MCP is the industry standard (Anthropic-led), supports any language, provides OS-level process isolation for free.

---

## Phase 2: Provider Control Plane

 --
**Goal**: Multi-provider routing, cost optimization, reliability.

 **Status**: Phase 2 Complete
- [x] Provider registry: OpenAI, Anthropic, Google, local (Ollama), custom endpoints
- [x] Fallback chains: primary → secondary → tertiary with automatic failover
- [x] Cost tracking: per-session, per-agent, per-model cost accumulation (SQLite-backed)
- [x] Prompt caching: Anthropic `system_and_3` strategy via OpenRouter `cache_control` in system prompt
- [x] Token counting: input/output tracking from OpenAI `usage` response
- [x] Rate limiting: per-provider token-bucket request throttling
- [ ] Model routing: route tasks to appropriate models (cheap model for simple tasks, powerful model for complex) — Phase 8
- [x] Provider health checks: automatic endpoint monitoring with probe

---

## Phase 3: Multi-Agent Orchestration

**Goal**: Subagent delegation, agent-to-agent communication, parallel execution.

**Status**: Phase 3 Complete (core delegation)

- [x] Subagent spawning: delegate tasks to child agents with scoped capabilities
- [x] Agent hierarchy: max depth 2 with depth counter
- [x] Capability inheritance: subagents get parent's full tool registry
- [x] Agent-to-agent messaging: structured SubagentRequest/SubagentResult protocol
- [x] Parallel agent execution: run independent subagents concurrently — `delegate_parallel` tool with `tokio::spawn`
- [ ] Agent lifecycle management: spawn, monitor, terminate — deferred
- [x] Result aggregation: collect and synthesize subagent outputs — aggregated in delegate_parallel
- [x] Implement `delegate` tool from Phase 1 stubs

---

## Phase 4: Memory & Skills

**Goal**: Persistent memory, cross-session search, self-improvement.

**Status**: Phase 4 Complete (core features)

- [x] Persistent memory: SQLite-backed key-value store surviving restarts
- [x] Cross-session search: FTS5 full-text search across all session history
- [ ] User modeling: per-user preferences, patterns, and context — deferred
- [x] Skill auto-creation: agents create reusable skills via the skills tool
- [ ] Progressive skill loading: 3-tier (category → list → full → file) for token efficiency — deferred
- [ ] Skill marketplace: share and discover community skills — deferred
- [ ] Self-nudging: agents proactively recall relevant past interactions — deferred
- [x] Iteration budget: configurable max iterations (default 90) with pressure warnings at 70%/90%
- [x] Implement `skills` tool from Phase 1 stubs

---

## Phase 5: Agent Completeness

**Goal**: Close critical functional gaps — search, task tracking, compression, cross-session recall.

**Status**: Phase 5 Complete

- [x] `search_files` tool: ripgrep-backed content search with path traversal protection
- [x] `todo` tool: session-scoped task list for multi-step work tracking
- [x] `clarify` tool: structured user questions with optional multiple-choice
- [x] `process` tool: background process management (list/register/kill)
- [x] FTS5 session search: full-text search across all conversation history
- [x] Context compression: 4-phase Hermes-style (prune → boundaries → summarize → reassemble)
- [x] `patch` tool: fuzzy file editing with 6 matching strategies (exact, prefix, suffix, contains, fuzzy, regex)

---

## Phase 6: Automation Platform

**Goal**: Scheduled tasks, command safety, prompt caching, MCP server mode.

**Status**: Phase 6 Complete

- [x] Cron scheduling: SQLite-backed `CronjobTool` with cron expressions, add/remove/list/tick
- [x] Command approval: regex-based dangerous pattern detection (`ApprovalChecker`)
- [x] Prompt caching: Anthropic/Gemini models via OpenRouter `cache_control` in system prompt
- [x] MCP server mode: `matrixclaw mcp-serve` exposing tools over JSON-RPC stdio
- [x] Plugin lifecycle hooks: `LifecycleHook` trait with `CompositeHook` dispatcher — Phase 6.5 complete

---

## Phase 6.5: Lifecycle Hooks

**Goal**: Extensible interception points for the agent loop — pre/post LLM calls, pre/post tool calls, session lifecycle.

**Status**: Phase 6.5 Complete

### Implementation

- [x] `HookPoint` enum: `PreLlmCall`, `PostLlmCall`, `PreToolCall`, `PostToolCall`, `OnSessionStart`, `OnSessionEnd`
- [x] `HookPayload` struct: typed event data (hook_point, session_id, tool_name, tool_arguments, tool_result, llm_response, iteration)
- [x] `HookAction` return type: `allow()` or `block(reason)` — blocks short-circuit the hook chain
- [x] `LifecycleHook` async trait: `on_event(&HookPayload) -> HookAction` + `name() -> &str`
- [x] `CompositeHook`: ordered dispatch over `Vec<Box<dyn LifecycleHook>>`, stops at first block
- [x] Agent loop integration: `run_prompt_with_policy` accepts `Option<&CompositeHook>` and dispatches at all six hook points
- [x] Pre-LLM call block terminates the run with the hook's reason
- [x] Pre-tool call block returns a `hooked: {reason}` tool result to the provider (does not execute the tool)
- [x] Full test coverage: allow/block/stop-at-first-block/empty-composite/serialization roundtrip

---

## Phase 7: Sandbox & Advanced

**Goal**: Isolated code execution, web search, browser automation, SSH sandbox.

**Status**: Phase 7 Complete

- [x] Docker sandbox backend: `DockerSandbox` with resource limits and automatic cleanup
- [x] `code_interpreter` tool: real implementation replacing stub, using Docker sandbox
- [x] `web_search` tool: real implementation with SearXNG backend replacing stub
- [x] Sandbox abstraction: `sandwrench` crate with `SandboxRuntime` trait, 4 backends, `SandboxProvider` factory
- [x] Browser automation: headless Chromium tools (navigate, screenshot, extract) behind `browser` feature flag
- [x] SSH sandbox backend: remote execution over SSH behind `ssh` feature flag
- [x] Iteration pressure warnings wired into chat mode

---

## Phase 7.5: Sandbox Abstraction (sandwrench)

**Goal**: Extract sandbox backends into a standalone `sandwrench` crate with a unified `SandboxRuntime` trait, multiple backend implementations, and config-driven backend selection.

**Status**: Phase 7.5 Complete

- [x] `SandboxRuntime` trait: `execute_code()` and `execute_command()` async methods
- [x] Docker backend: wraps existing `DockerSandbox` behind `SandboxRuntime` (default)
- [x] E2B backend: cloud microVM integration for ephemeral sandboxes
- [x] Daytona backend: self-hosted workspace API integration
- [x] Local backend: passthrough execution on host (dev/test only)
- [x] `SandboxProvider` factory: reads `~/.matrixclaw/config/sandbox.json` and constructs the selected backend
- [x] `code_interpreter` tool updated to request sandbox via `SandboxProvider` instead of direct Docker dependency
- [x] Config schema: `sandbox.json` with per-backend settings (image, memory, timeout, API keys)

---

## Phase 8: Differentiation

**Goal**: Profiles, self-evolving skills, messaging gateway, model routing.

**Status**: Phase 8 In Progress

- [x] Agent lifecycle management: `SubagentTracker` with `agent_list` and `agent_cancel` tools
- [x] Progressive skill loading: categories, search, enhanced list with summaries, frontmatter parsing
- [x] Self-nudging engine: `NudgeEngine` + `MemoryNudgeStore` for context injection
- [x] Messaging gateway: `MessageGateway` trait, `AgentBridge`, stub adapters (Matrix/Discord/Telegram/Slack), `gateway-serve` CLI
- [ ] Multi-instance profiles: per-agent configuration with scoped capabilities
- [x] Self-evolving skills: Rust-native GePA+MiProv2 with TraceCollector, TraceAnalyzer, SkillRewriter
- [ ] Model routing: automatic task-to-model assignment

---

## Architecture Diagram

```
┌──────────────────────────────────────────────────────┐
│                    matrixclaw binary                   │
├──────────────────────────────────────────────────────┤
│  Interfaces                                           │
│  ├── TUI Chat (readline + streaming)                  │
│  ├── HTTP/SSE API (tiny_http)                         │
│  ├── MCP Server (stdio JSON-RPC)           [Phase 6]  │
│  ├── Cron Scheduler                        [Phase 6]  │
│  └── Messaging Gateway                     [Phase 8]  │
├──────────────────────────────────────────────────────┤
│  Agent Core (agent-core)                              │
│  ├── Async ReAct loop       ├── Policy engine         │
│  ├── JSON function-calling  ├── Iteration budget      │
│  ├── Lifecycle Hooks (Phase 6.5)                      │
│  │   ├── HookPoint (Pre/Post LLM + Tool, Session)     │
│  │   ├── CompositeHook dispatcher                      │
│  │   └── HookAction (allow / block)                    │
│  ├── Context compression          [Phase 5]           │
│  └── Command approval             [Phase 6]           │
├──────────────────────────────────────────────────────┤
│  Tool System (matrixclaw-tools)                       │
│  ├── Tool Registry               ├── 30+ Built-in     │
│  │   ├── filesystem (read/write/edit/list/search/patch)│
│  │   ├── terminal + process                           │
│  │   ├── web (fetch + search)       [Phase 7]         │
│  │   ├── memory (SQLite + search)                     │
│  │   ├── skills (list/read/create + progressive) [P8] │
│  │   ├── delegate + delegate_parallel                 │
│  │   ├── agent_list + agent_cancel [Phase 8]          │
│  │   ├── todo + clarify + session_search [Phase 5]    │
│  │   ├── code_interpreter            [Phase 7]        │
│  │   ├── browser automation          [Phase 7]        │
│  │   ├── cronjob                     [Phase 6]        │
│  │   └── nudge_store                 [Phase 8]        │
│  └── MCP Client + Server            [Phase 6]         │
├──────────────────────────────────────────────────────┤
│  Provider Plane (provider-plane)                      │
│  ├── Provider Registry   ├── Fallback Chains          │
│  ├── Cost Tracking       ├── Rate Limiting            │
│  ├── Health Checks       └── Prompt Caching [Phase 6] │
├──────────────────────────────────────────────────────┤
│  Session Runtime (session-runtime)                    │
│  ├── SQLite Storage (FTS5)  ├── Queue & Compaction    │
│  ├── Recovery               ├── Message Projection    │
│  └── Context Compression              [Phase 5]       │
├──────────────────────────────────────────────────────┤
│  Sandbox Backends (sandwrench)              [Phase 7] │
│  ├── Docker (default)    ├── E2B (cloud microVM)       │
│  ├── Daytona (self-host) ├── SSH (russh)    [Phase 7]  │
│  └── Local (passthrough/dev)                        │
└──────────────────────────────────────────────────────┘
```

## Competitive Landscape

### vs Hermes Agent (NousResearch)
- **Their edge**: 40+ tools, 15+ platforms, Python ecosystem, self-improving skills, RL training
- **Our edge**: Single-binary Rust, multi-agent profiles, provider control plane, no Python dependency
- **Strategy**: Match tool count by Phase 2, match skill system by Phase 4, differentiate on runtime simplicity

### vs Claude Code / Codex / Cursor
- **Their edge**: Deep IDE integration, large user bases, massive model backing
- **Our edge**: Self-hosted, cost-controlled, multi-agent, extensible tool system
- **Strategy**: Target ops/automation workloads, not just coding assistance
