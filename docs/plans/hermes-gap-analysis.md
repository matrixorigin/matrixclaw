# MatrixClaw v2 Roadmap — Closing the Gap with Hermes

**Date**: 2026-04-03
**Status**: Phase 4 Complete → Rewriting Phase 5+ to close competitive gaps

This document supersedes the Phase 5 section of `runtime-rethink.md`. It maps Hermes Agent's key features to MatrixClaw implementation tasks, prioritized by impact.

---

## Priority Matrix

| Priority | Feature | Hermes Has | Our Status | Impact |
|----------|---------|------------|------------|--------|
| P0 | `search_files` tool (ripgrep) | Yes (`search_files`) | Missing | Critical for codebase navigation |
| P0 | Context compression | Yes (4-phase LLM summarization) | Session compaction only | Critical for long sessions |
| P0 | FTS5 session search | Yes (`session_search` + FTS5) | Missing | Critical for cross-session recall |
| P0 | `todo` tool (task tracking) | Yes (`todo`) | Missing | Essential for multi-step tasks |
| P0 | `clarify` tool (user questions) | Yes (`clarify`) | Missing | Essential for interactive agents |
| P1 | `process` tool (background jobs) | Yes (`process`) | Missing | Important for long-running tasks |
| P1 | `patch` tool (fuzzy file editing) | Yes (`patch`, 9 strategies) | `edit_file` (find/replace) | Important for code editing |
| P1 | Command approval system | Yes (regex + LLM auto-approve) | `ToolPreflightPolicy` (basic) | Important for safety |
| P1 | Cron scheduling | Yes (`cronjob`) | Missing | Turns agent into automation platform |
| P1 | MCP server mode | Yes (`hermes mcp serve`) | Client only | Important for ecosystem |
| P1 | Prompt caching (Anthropic) | Yes (`system_and_3` strategy) | Missing | ~75% cost reduction |
| P2 | Docker sandbox backend | Yes (6 backends) | Missing | Important for security |
| P2 | `code_interpreter` tool | Yes (`execute_code`) | Stub | Enables complex workflows |
| P2 | Browser automation | Yes (11 browser tools) | Missing | Enables web interaction |
| P2 | Web search (real) | Yes (Exa/Tavily) | Stub | Important for research tasks |
| P2 | Plugin lifecycle hooks | Yes (4 hooks) | Missing | Extensibility |
| P3 | Profiles (multi-instance) | Yes (`hermes profile`) | Missing | Nice-to-have |
| P3 | Voice/messaging platforms | Yes (6 platforms) | TUI + HTTP only | Different market |
| P3 | Self-evolving skills (DSPy) | Yes (`hermes-agent-self-evolution`) | Manual skills | Research project |
| P3 | RL training pipeline | Yes (Atropos) | Missing | Different market |

---

## Phase 5: Agent Completeness (P0 items)

**Goal**: Close the most critical functional gaps. An agent without search, task tracking, compression, and cross-session recall is fundamentally limited.

### Stage 5.1 — `search_files` Tool

Add ripgrep-backed content search to `matrixclaw-tools`.

- New tool: `search_files` in `crates/matrixclaw-tools/src/builtin/search_files.rs`
- Parameters: `pattern` (regex), `path` (optional subdir), `include` (file glob, optional), `max_results` (default 50)
- Implementation: shell out to `rg` (ripgrep) since it's universally available on dev machines
- Returns: matched file paths with line numbers and matched line content
- Replaces the need for agents to chain `list_directory` + `read_file` to find code

### Stage 5.2 — `todo` Tool

Session-scoped task list for multi-step work.

- New tool: `todo` in `crates/matrixclaw-tools/src/builtin/todo.rs`
- Parameters: `action` (list/add/update/remove), `id` (optional), `text` (optional), `status` (pending/in_progress/done)
- Storage: in-memory `Vec<TodoItem>` (per-session, not persistent across restarts)
- Hermes pattern: agents use todo to track progress on 3+ step tasks; the model can see its own task list and update it

### Stage 5.3 — `clarify` Tool

Structured user interaction for disambiguation.

- New tool: `clarify` in `crates/matrixclaw-tools/src/builtin/clarify.rs`
- Parameters: `question` (required), `options` (optional array of up to 4 choices)
- Returns the user's answer as the tool result
- **Challenge**: This is a blocking tool that needs user input mid-agent-loop. Implementation:
  - In chat mode: pause the loop, print the question, wait for stdin input, return as tool result
  - In HTTP API mode: return a special "needs clarification" event, wait for client response
- Simplest first pass: chat-mode only, use `std::io::stdin()` in the tool's execute method

### Stage 5.4 — Context Compression

Hermes-style 4-phase compression for long conversations.

- Modify: `crates/session-runtime/` — add `ContextCompressor`
- Algorithm:
  1. Prune old tool results (>200 chars → placeholder)
  2. Determine head/middle/tail boundaries
  3. LLM-generate structured summary (Goal, Progress, Key Decisions, Next Steps)
  4. Reassemble: head + summary + tail
- Trigger: when token count exceeds configurable threshold (default: 50% of context window)
- Requires: passing provider reference to the compression engine (same callback pattern as delegate tool)
- **Key insight from Hermes**: iterative re-compression — pass previous summary to LLM for updating rather than re-summarizing

### Stage 5.5 — FTS5 Session Search

Full-text search across all session history.

- Modify: `crates/session-runtime/src/sqlite.rs` — add FTS5 virtual table
- Schema: `CREATE VIRTUAL TABLE messages_fts USING fts5(content, content=messages, content_rowid=id)`
- New tool: `session_search` in `crates/matrixclaw-tools/src/builtin/session_search.rs`
- Parameters: `query` (required), `limit` (default 10)
- Returns: matching messages with session ID, timestamp, and content snippet
- The tool needs access to the session database path (same pattern as memory tool)

### Stage 5.6 — Tooling Polish

- Improve `edit_file` with fuzzy matching (Hermes has 9 strategies)
- Add `process` tool for background process management (list, wait, kill)
- Wire iteration pressure warnings into chat mode display

---

## Phase 6: Automation Platform (P1 items)

**Goal**: Turn MatrixClaw from a chat agent into an automation platform.

### Stage 6.1 — Cron Scheduling

Built-in task scheduler.

- New crate or module: `crates/app-host/src/cron.rs`
- Jobs stored in SQLite: `~/.matrixclaw/state/cron.sqlite3`
- Natural language → cron expression (LLM-assisted)
- Job execution: spawn fresh session, run agent with task prompt, deliver result
- Delivery: print to CLI, or POST to webhook URL
- Config: `~/.matrixclaw/config/cron.json`

### Stage 6.2 — MCP Server Mode

Expose MatrixClaw sessions/tools to external MCP clients (Claude Code, Cursor, etc.).

- New module: `crates/app-host/src/mcp_server.rs`
- Implements MCP server protocol (JSON-RPC over stdio)
- Exposes tools: `conversations_list`, `messages_read`, `messages_send`, `tools_list`, `tools_call`
- Command: `matrixclaw mcp-serve`

### Stage 6.3 — Command Approval System

Hermes-style dangerous command approval.

- Enhance: `crates/agent-core/src/policy.rs`
- Add `CommandApprovalPolicy` that checks tool calls against regex patterns
- Patterns: `rm -rf`, `mkfs`, `curl|sh`, fork bombs, etc.
- CLI: interactive prompt (approve/deny/allow permanently)
- Config: `~/.matrixclaw/config/approvals.json`

### Stage 6.4 — Prompt Caching

Anthropic-style prompt caching for cost reduction.

- Enhance: `crates/provider-plane/src/openai.rs` — add Anthropic caching headers
- Strategy: `system_and_3` — 4 cache breakpoints (system prompt + rolling 3-message window)
- Auto-enable for Claude models via Anthropic or OpenRouter
- ~75% input token cost reduction on multi-turn conversations

### Stage 6.5 — Plugin Lifecycle Hooks

Extensibility for custom behavior.

- Define hook points: `pre_llm_call`, `post_llm_call`, `on_session_start`, `on_session_end`, `pre_tool_call`, `post_tool_call`
- Hooks are MCP servers configured in `~/.matrixclaw/config/hooks.json`
- Each hook receives event payload via MCP notification, can modify behavior

---

## Phase 7: Sandbox & Advanced (P2 items)

### Stage 7.1 — Docker Sandbox Backend

Containerized tool execution.

- New module: `crates/matrixclaw-tools/src/sandbox/`
- Docker backend: create container, mount workspace, execute command, capture output, destroy container
- Security: read-only root, dropped capabilities, no privilege escalation, PID limits
- Config: `~/.matrixclaw/config/sandbox.json`

### Stage 7.2 — Code Interpreter

Safe Python/Rust execution environment.

- Replace `code_interpreter` stub with real implementation
- Uses Docker sandbox backend
- Supports: Python and Rust (via cargo script)
- Returns: stdout, stderr, exit code
- Resource limits: CPU time, memory, disk

### Stage 7.3 — Browser Automation

Web interaction via accessibility tree.

- New tools: `browser_navigate`, `browser_snapshot`, `browser_click`, `browser_type`, `browser_press`, `browser_scroll`
- Implementation: headless Chromium via `chromiumoxide` or shell out to `playwright`
- Compact accessibility-tree snapshots (not full DOM)

### Stage 7.4 — Real Web Search

Replace `web_search` stub with actual search.

- Backend options: Exa API, Tavily API, SearXNG (self-hosted)
- Returns: titles, URLs, descriptions (up to 5 results)
- Config: `~/.matrixclaw/config/search.json`

---

## Phase 8: Differentiation (P3 items)

These are where we differentiate rather than catch up.

### Stage 8.1 — Profiles (Multi-Instance)

Multiple isolated MatrixClaw instances from one binary.

- Command: `matrixclaw profile create <name>`
- Each profile: own config, memory, sessions, skills
- Shared: provider plane, binary

### Stage 8.2 — Self-Evolving Skills

DSPy/GEPA-style skill prompt optimization.

- Agents create skills from successful task patterns
- Skills self-improve via A/B testing on subsequent uses
- Track skill effectiveness metrics

### Stage 8.3 — Messaging Gateway

Multi-platform messaging (Telegram, Discord, etc.).

- This is a large undertaking. Evaluate whether to build or integrate with existing gateway solutions.
- Matrix protocol support is already partially there (`matrixclaw-compat-openclaw`)

---

## Updated Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    matrixclaw binary                      │
├─────────────────────────────────────────────────────────┤
│  Interfaces                                              │
│  ├── TUI Chat (readline + streaming)                     │
│  ├── HTTP/SSE API (tiny_http)                            │
│  ├── MCP Server (stdio JSON-RPC)              [Phase 6]  │
│  └── Cron Scheduler                          [Phase 6]  │
├─────────────────────────────────────────────────────────┤
│  Agent Core (agent-core)                                 │
│  ├── Async ReAct loop          ├── Policy engine         │
│  ├── JSON function-calling     ├── Iteration budget      │
│  ├── Context compression       [Phase 5]                 │
│  └── Command approval          [Phase 6]                 │
├─────────────────────────────────────────────────────────┤
│  Tool System (matrixclaw-tools)                          │
│  ├── Tool Registry              ├── 18+ Built-in Tools   │
│  │   ├── filesystem (read/write/edit/list/search)        │
│  │   ├── terminal + process          [Phase 5]           │
│  │   ├── web (fetch + search)        [Phase 7]           │
│  │   ├── memory (SQLite + search)                         │
│  │   ├── skills (list/read/create)                        │
│  │   ├── delegate (subagent spawning)                     │
│  │   ├── todo (task tracking)        [Phase 5]           │
│  │   ├── clarify (user questions)    [Phase 5]           │
│  │   ├── session_search (FTS5)       [Phase 5]           │
│  │   ├── code_interpreter            [Phase 7]           │
│  │   ├── browser automation          [Phase 7]           │
│  │   └── cron management             [Phase 6]           │
│  └── MCP Client + Server                       [Phase 6] │
├─────────────────────────────────────────────────────────┤
│  Provider Plane (provider-plane)                         │
│  ├── Provider Registry   ├── Fallback Chains             │
│  ├── Cost Tracking       ├── Rate Limiting               │
│  ├── Health Checks       └── Prompt Caching   [Phase 6]  │
├─────────────────────────────────────────────────────────┤
│  Session Runtime (session-runtime)                       │
│  ├── SQLite Storage (FTS5)     ├── Queue & Compaction    │
│  ├── Recovery                  ├── Message Projection    │
│  └── Context Compression                  [Phase 5]      │
├─────────────────────────────────────────────────────────┤
│  Sandbox Backends                              [Phase 7] │
│  ├── Docker    ├── SSH      └── Local                    │
└─────────────────────────────────────────────────────────┘
```

---

## What We Keep That Hermes Doesn't Have

1. **Single binary, zero dependencies** — Hermes needs Python + Node.js + pip
2. **Typed provider control plane** — Rust type system for provider config, not YAML stringly-typed
3. **Cost tracking built-in** — per-session, per-model cost accumulation in SQLite
4. **Rate limiting built-in** — per-provider token-bucket
5. **Health checks built-in** — automatic endpoint monitoring
6. **No GIL** — Rust async runtime vs Python's GIL for concurrent tool execution

## What We're Adopting From Hermes

1. **Context compression** — their 4-phase algorithm is excellent
2. **FTS5 session search** — table-stakes for any serious agent
3. **Progressive skill loading** — 3-tier for token efficiency
4. **Command approval** — regex + optional LLM auto-approve
5. **Cron scheduling** — natural-language scheduled tasks
6. **MCP server mode** — expose to Claude Code / Cursor
7. **Tool availability checking** — dynamic schema patching to prevent hallucination of unavailable tools
