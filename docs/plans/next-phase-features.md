# Implementation Plan: Next Phase Features

**Date**: 2026-04-07
**Status**: Planning

---

## Plan 1: SSH Sandbox Backend (Phase 7)

**Goal**: Add SSH-based remote sandbox backend to sandwrench, enabling code execution on remote machines.

**Files to create/modify**:
- `crates/sandwrench/src/backend/ssh.rs` — new backend
- `crates/sandwrench/src/backend/mod.rs` — add `pub mod ssh`
- `crates/sandwrench/src/config.rs` — add `SshConfig` fields
- `crates/sandwrench/Cargo.toml` — add `russh` dependency

**Implementation**:

### `SshSandboxBackend`

```rust
pub struct SshSandboxBackend {
    host: String,
    port: u16,
    username: String,
    auth: SshAuth,
    config: SandboxConfig,
}

pub enum SshAuth {
    Password(String),
    Key { path: PathBuf, passphrase: Option<String> },
    Agent,
}
```

Implements `SandboxRuntime`:
- `execute_code()`: SSH to remote, run `echo '<code>' | <interpreter>` via channel exec
- `execute_command()`: SSH to remote, run command via channel exec
- `is_available()`: attempt TCP connect to host:port with 2s timeout
- `supported_languages()`: depends on remote (default: python, bash, node)

**Config** (`sandbox.json`):
```json
{
  "kind": "ssh",
  "ssh": {
    "host": "sandbox.example.com",
    "port": 22,
    "username": "sandbox",
    "auth": "agent",
    "working_dir": "/tmp/sandbox"
  }
}
```

**Dependency**: `russh` crate (pure Rust SSH2 client, async, no C deps)

**Tests** (no live SSH needed):
- Config serialization roundtrip
- Auth enum serialization
- Code building command strings (unit test the shell escaping)
- `is_available()` returns false on unreachable host

---

## Plan 4: Agent Lifecycle Management (deferred)

**Goal**: Track subagent state (spawned, running, completed, failed) with ability to monitor and terminate.

**Files to create/modify**:
- `crates/agent-core/src/subagent.rs` — new module
- `crates/agent-core/src/lib.rs` — add `pub mod subagent`
- `crates/matrixclaw-tools/src/builtin/delegate.rs` — integrate lifecycle
- `crates/matrixclaw-tools/src/builtin/delegate_parallel.rs` — integrate lifecycle

**Implementation**:

### `SubagentHandle`

```rust
pub struct SubagentHandle {
    pub id: String,
    pub task: String,
    pub status: SubagentStatus,
    pub started_at: Instant,
    pub completed_at: Option<Instant>,
    pub result: Option<SubagentResult>,
}

pub enum SubagentStatus {
    Spawned,
    Running,
    Completed,
    Failed(String),
    Cancelled,
}
```

### `SubagentTracker`

```rust
pub struct SubagentTracker {
    agents: Arc<Mutex<HashMap<String, SubagentHandle>>>,
    cancellation: Arc<Mutex<HashMap<String, CancellationToken>>>,
}

impl SubagentTracker {
    pub fn spawn(&self, task: String) -> String;     // returns agent ID
    pub fn complete(&self, id: &str, result: SubagentResult);
    pub fn fail(&self, id: &str, error: String);
    pub fn cancel(&self, id: &str);                   // triggers CancellationToken
    pub fn list(&self) -> Vec<SubagentHandle>;         // snapshot of all agents
    pub fn get(&self, id: &str) -> Option<SubagentHandle>;
}
```

### Integration with delegate tools

- `delegate` tool registers in tracker before calling runner, deregisters on completion
- `delegate_parallel` registers each subtask separately
- New `agent_list` tool: lists all active subagents and their status
- New `agent_cancel` tool: cancels a running subagent by ID

**Tests**:
- Tracker spawn/complete lifecycle
- Cancel sets status to Cancelled
- List returns all agents
- Parallel tool registers N agents
- Cancel during execution returns Cancelled status

---

## Plan 6: Progressive Skill Loading (deferred)

**Goal**: Load skills in 3 tiers to save tokens — category list → skill names → full content.

**Files to modify**:
- `crates/matrixclaw-tools/src/builtin/skills.rs` — add tier support

**Implementation**:

### Tier 1: Category listing

```
action: "categories"
→ returns: list of skill directories/categories with counts
```

Scans `~/.matrixclaw/skills/` and groups by subdirectory or frontmatter `category:` field.

### Tier 2: Skill listing (existing, enhanced)

```
action: "list"
→ returns: skill names + first line (title/summary) + category
```

For each `.md` file in skills dir, read first line only (title). Returns compact table.

### Tier 3: Full content (existing)

```
action: "read", name: "my-skill"
→ returns: full Markdown content
```

Unchanged from current behavior.

### New action: `search`

```
action: "search", query: "deployment"
→ returns: skill names whose title/tags match the query (Tier 2 format)
```

Searches frontmatter tags and first-line titles.

### Skill frontmatter (optional)

Skills can have YAML frontmatter:
```markdown
---
category: deployment
tags: [docker, k8s, ci-cd]
summary: Deploy to Kubernetes with rollouts
---
# Deploy to K8s
...full content...
```

If no frontmatter, infer category from subdirectory name.

**Tests**:
- Categories action returns grouped skills
- List action returns compact summaries
- Search action matches by tag/title
- Skills without frontmatter get inferred category
- Tier 1 output is smaller than Tier 2, Tier 2 smaller than Tier 3

---

## Plan 8: Self-Nudging (deferred)

**Goal**: Agent proactively recalls relevant past interactions from memory when starting new tasks.

**Files to create/modify**:
- `crates/agent-core/src/nudge.rs` — new module
- `crates/agent-core/src/lib.rs` — add `pub mod nudge`
- `crates/agent-core/src/loop.rs` — inject nudges before first LLM call

**Implementation**:

### `NudgeEngine`

```rust
pub struct NudgeEngine {
    memory: Arc<dyn NudgeStore>,
    threshold: f64,
}

pub trait NudgeStore: Send + Sync {
    fn search_relevant(&self, query: &str, limit: usize) -> Vec<NudgeEntry>;
}

pub struct NudgeEntry {
    pub topic: String,
    pub content: String,
    pub relevance: f64,
    pub timestamp: DateTime<Utc>,
}
```

### How it works

1. When a new `RunRequest` starts, extract keywords from the prompt
2. Search memory tool's SQLite for relevant past interactions
3. If relevance >= threshold (default 0.6), inject as context:
   ```
   [Relevant context from past interactions]:
   - <topic>: <summary>
   ```
4. This happens inside the agent loop, before the first LLM call, using the PreLlmCall hook point

### Integration

- Uses the existing `memory` tool's SQLite backend as the `NudgeStore`
- Injected via `LifecycleHook` (PreLlmCall) — no changes to loop.rs needed
- The hook reads the prompt from `HookPayload`, queries memory, and returns `allow()` after injecting context

**Tests**:
- NudgeEngine finds relevant entries above threshold
- Entries below threshold are skipped
- Empty memory returns no nudges
- Context injection format is correct
- Multiple relevant entries are concatenated

---

## Plan 10: Messaging Gateway (Phase 8)

**Goal**: Connect MatrixClaw to messaging platforms (Matrix, Discord, Slack, Telegram) as a bot.

**Files to create/modify**:
- `crates/app-host/src/gateway/mod.rs` — gateway trait + registry
- `crates/app-host/src/gateway/matrix.rs` — Matrix adapter
- `crates/app-host/src/gateway/discord.rs` — Discord adapter
- `crates/app-host/src/gateway/slack.rs` — Slack adapter
- `crates/app-host/src/gateway/telegram.rs` — Telegram adapter
- `crates/app-host/src/lib.rs` — new `gateway-serve` subcommand
- `crates/app-host/Cargo.toml` — add `matrix-sdk`, `serenity` (Discord), `teloxide` (Telegram) behind feature flags

**Implementation**:

### Gateway Trait

```rust
#[async_trait]
pub trait MessageGateway: Send + Sync {
    async fn start(&self, handler: Box<dyn MessageHandler>) -> Result<(), String>;
    async fn send(&self, channel: &str, message: &str) -> Result<(), String>;
    fn platform_name(&self) -> &str;
}

pub trait MessageHandler: Send + Sync {
    async fn on_message(&self, msg: IncomingMessage) -> OutgoingMessage;
}

pub struct IncomingMessage {
    pub platform: String,
    pub channel: String,
    pub sender: String,
    pub content: String,
    pub thread_id: Option<String>,
}

pub struct OutgoingMessage {
    pub content: String,
    pub thread_id: Option<String>,
}
```

### Agent Bridge

The `MessageHandler` implementation:
1. Receives `IncomingMessage`
2. Creates a `LiveRunRequest` with the message as prompt
3. Runs through `run_with_provider_and_queue_stream`
4. Collects the final message
5. Returns `OutgoingMessage` with the response

Each platform/channel gets its own session ID for context persistence.

### Matrix Adapter

Uses `matrix-sdk` crate:
- Listens on configured room
- Supports threaded conversations (session per thread)
- Supports commands: `!ask <question>`, `!reset` (clear session)

### Discord Adapter

Uses `serenity` crate:
- Listens on configured channel(s)
- Supports slash commands: `/ask`, `/reset`
- Thread-based sessions

### Slack Adapter

Uses `slack-morphism` or raw HTTP:
- Listens via Socket Mode (WebSocket)
- Supports app mentions and DMs
- Thread-based sessions

### Telegram Adapter

Uses `teloxide` crate:
- Listens via long polling
- Supports private chats and group mentions
- Per-chat session IDs

### CLI

```
matrixclaw gateway-serve --platform matrix --config ~/.matrixclaw/config/gateway.json
matrixclaw gateway-serve --platform discord
matrixclaw gateway-serve --platform telegram
```

### Config (`gateway.json`)

```json
{
  "platforms": {
    "matrix": {
      "homeserver": "https://matrix.org",
      "access_token": "${MATRIX_TOKEN}",
      "rooms": ["!roomid:matrix.org"]
    },
    "discord": {
      "token": "${DISCORD_TOKEN}",
      "channels": ["123456789"]
    },
    "telegram": {
      "token": "${TELEGRAM_TOKEN}",
      "allowed_chats": [12345]
    },
    "slack": {
      "app_token": "${SLACK_APP_TOKEN}",
      "bot_token": "${SLACK_BOT_TOKEN}"
    }
  },
  "model": "moonshotai/kimi-k2.5",
  "max_message_length": 4000
}
```

**Feature flags**: Each platform behind its own feature flag (`gateway-matrix`, `gateway-discord`, `gateway-telegram`, `gateway-slack`) to avoid pulling in all SDKs.

**Tests**:
- `IncomingMessage`/`OutgoingMessage` serialization
- Gateway trait mock (test the handler bridge)
- Session ID derivation per platform/channel
- Config parsing with env var interpolation
- Message truncation for platform limits

---

## Execution Priority

| Order | Plan | Depends on | Estimated LOC |
|-------|------|-----------|---------------|
| 1 | SSH Sandbox | sandwrench (done) | ~200 |
| 2 | Agent Lifecycle | delegate tools (done) | ~300 |
| 3 | Progressive Skills | skills tool (done) | ~150 |
| 4 | Self-Nudging | memory tool + hooks (done) | ~200 |
| 5 | Messaging Gateway | app-host + chat (done) | ~800 |
