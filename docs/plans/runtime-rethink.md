# MatrixClaw Runtime Rethink — 5-Phase Roadmap

**Date**: 2026-03-31
**Status**: Phase 5 Complete — Phase 6 Next
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
- [ ] Prompt caching: Anthropic `system_and_3` strategy (~75% input cost reduction) — deferred, requires native Anthropic provider
- [x] Token counting: input/output tracking from OpenAI `usage` response
- [x] Rate limiting: per-provider token-bucket request throttling
- [ ] Model routing: route tasks to appropriate models (cheap model for simple tasks, powerful model for complex) — deferred
- [x] Provider health checks: automatic endpoint monitoring with probe

---

## Phase 3: Multi-Agent Orchestration

**Goal**: Subagent delegation, agent-to-agent communication, parallel execution.

**Status**: Phase 3 Complete (core delegation)

- [x] Subagent spawning: delegate tasks to child agents with scoped capabilities
- [x] Agent hierarchy: max depth 2 with depth counter
- [x] Capability inheritance: subagents get parent's full tool registry
- [x] Agent-to-agent messaging: structured SubagentRequest/SubagentResult protocol
- [ ] Parallel agent execution: run independent subagents concurrently — deferred
- [ ] Agent lifecycle management: spawn, monitor, terminate — deferred
- [ ] Result aggregation: collect and synthesize subagent outputs — deferred
- [x] Implement `delegate` tool from Phase 1 stubs

---

## Phase 4: Memory & Skills

**Goal**: Persistent memory, cross-session search, self-improvement.

**Status**: Phase 4 Complete (core features)

- [x] Persistent memory: SQLite-backed key-value store surviving restarts
- [ ] Cross-session search: FTS5 full-text search across all session history — deferred
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
- [ ] `patch` tool: fuzzy file editing with multiple matching strategies — deferred
- [x] Iteration pressure warnings wired into chat mode

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────┐
│                  matrixclaw binary               │
├─────────────────────────────────────────────────┤
│  HTTP/SSE API (tiny_http)                       │
│  ├── Agent API    ├── Session API               │
│  ├── Skills API   ├── MCP API                   │
│  └── Gateway API  └── Queue API                 │
├─────────────────────────────────────────────────┤
│  Agent Core (agent-core)                        │
│  ├── Async ReAct loop    ├── Policy engine      │
│  ├── JSON function-calling   └── Streaming      │
├─────────────────────────────────────────────────┤
│  Tool System (matrixclaw-tools)                 │
│  ├── Tool Registry      ├── 13 Built-in Tools   │
│  └── MCP Client (future plugin path)            │
├─────────────────────────────────────────────────┤
│  Provider Plane (Phase 2)                       │
│  ├── Provider Registry   ├── Fallback Chains    │
│  ├── Cost Tracking       └── Prompt Caching     │
├─────────────────────────────────────────────────┤
│  Session Runtime (session-runtime)              │
│  ├── SQLite Storage     ├── Queue & Compaction  │
│  ├── Recovery           └── Message Projection  │
├─────────────────────────────────────────────────┤
│  Sandbox Backends (Phase 5)                     │
│  ├── Docker    ├── SSH      └── Local           │
└─────────────────────────────────────────────────┘
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
