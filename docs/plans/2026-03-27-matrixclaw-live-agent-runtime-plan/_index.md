# MatrixClaw Live Agent Runtime Completion Plan

> **For Claude:** REQUIRED SUB-SKILL: Load `superpowers:executing-plans` skill using the Skill tool to implement this plan task-by-task.

**Goal:** Finish the next product-critical slice by replacing the current minimal direct provider call with a session-backed, streaming-capable live runtime that serves the browser UI and the OpenClaw transport adapter from the same execution core.

**Architecture:** Keep `matrixclaw-agent-core` as the streaming-first loop kernel, but stop bypassing it at the product boundary. `app-host` should expose one live runtime service that owns provider selection, session persistence, queue delivery, tool lifecycle, and event projection. The browser workspace and `compat-openclaw` should both speak to that shared service instead of running separate ad hoc execution paths. Because the current `tiny_http` loop is sufficient for static pages but weak for long-lived streaming and shared runtime state, this plan allows the loopback server boundary to evolve as needed during the session-backed chat endpoint work rather than treating the current server implementation as fixed.

## Terminology

This plan uses the following layer names deliberately:

- **live runtime service**
  - the single internal layer that owns sessions, prompts, queue delivery, tool lifecycle, provider calls, transcript persistence, and replayable runtime events
- **transport adapter**
  - a boundary that receives requests from one client surface and translates them into the live runtime service contract
  - examples in current scope:
    - browser loopback HTTP for the local MatrixClaw web UI
    - OpenClaw-compatible HTTP/WebSocket transport in `compat-openclaw`
- **gateway**
  - reserved for future ingress from external messengers, IM systems, or agent-to-agent delivery surfaces
  - examples out of scope for this plan:
    - Telegram, Slack, Discord, Matrix, WhatsApp, or email connectors
    - multi-agent inbox/outbox routing across channels

The rule is:

- browser UI and OpenClaw are not separate runtime layers
- they are separate transport adapters
- both must feed the same internal live runtime service
- future messenger gateways can be added on top of that same runtime service without forking runtime behavior

Generalization requirements:

- external channel contracts should be normalized before they enter the runtime core
- messenger-specific quirks must stay in adapter or gateway code
- the live runtime service must not learn about Slack threads, Telegram chats, Discord channels, Matrix rooms, or similar channel-native concepts directly
- adapter and gateway layers should translate those concepts into stable internal envelopes for:
  - sender identity
  - channel or thread identity
  - agent target identity
  - text and attachment payloads
  - reply routing metadata
  - delivery outcome and retry state
- adding a new external system should mostly mean adding a new adapter or gateway, not changing the runtime core contract

**Tech Stack:** Rust stable, Cargo workspace, `matrixclaw-agent-core`, `matrixclaw-session-runtime`, `matrixclaw-compat-openclaw`, `reqwest` for provider HTTP, loopback HTTP/SSE routing in `app-host`, `SvelteKit`, `pnpm`, Playwright, OpenRouter live smoke with `moonshotai/kimi-k2.5`

**Design Support:**
- [BDD Specs](../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md)
- [Architecture](../2026-03-26-rust-openclaw-runtime-design/architecture.md)
- [Runtime Model](../2026-03-26-rust-openclaw-runtime-design/runtime-model.md)
- [Protocol Compatibility](../2026-03-26-rust-openclaw-runtime-design/protocol-compatibility.md)
- [Delivery Plan](../2026-03-26-rust-openclaw-runtime-design/delivery-plan.md)
- [Web UI Plan](../2026-03-27-matrixclaw-web-ui-plan/_index.md)

## Context

MatrixClaw now has a real browser shell, real loopback pages, queue controls, skills inventory, screenshot smoke, and a minimal OpenRouter-backed prompt path. That is enough to prove packaging and local UI integration. It is not enough to claim a true agent runtime. The critical missing behavior is that the live chat path still bypasses the durable session runtime and the richer agent loop semantics that the design package was written around.

This creates an architectural mismatch:

- the design says the product should be session-backed, event-driven, and shared across transport adapters
- the current live path is a direct provider call that returns one final message
- the UI can send a real prompt, but the send path does not yet own session ids, streaming deltas, queue delivery, tool lifecycle, or resume-after-restart behavior

That mismatch is now the highest-value next step. Until it is corrected, the project will look more complete than it really is.

| Aspect | Current State | Target State |
|--------|--------------|--------------|
| Provider path | Minimal OpenRouter adapter, one request in and one final message out | Reusable provider service with normalized streaming events and fixture-backed tests |
| Live chat endpoint | [`/api/agent/run`](/home/momo/src/matrixclaw/crates/app-host/src/http/agent_api.rs) directly calls provider and returns a message | Session-backed run service creates/continues sessions, persists transcript, and returns session-aware runtime events |
| Browser transcript | Workspace composer appends a final assistant message only | Browser renders ordered deltas, finalization, metadata, and replay-safe transcript state |
| Queue behavior | Queue UI works independently, but live prompt execution does not consume queue semantics | Steering and follow-up participate in real session runtime delivery rules |
| Resume behavior | Session-runtime tests exist, but browser/API path does not expose session continuation | HTTP/browser/compat paths can reopen a prior session and continue cleanly |
| Tool lifecycle | Tool loop exists in isolated `agent-core` tests | Live provider path executes tools, persists results, and surfaces lifecycle events |
| Safety surface | Blocked-tool semantics exist at the core design level | Live path persists and renders blocked-tool results without crashing or hiding them |
| Transport adapters | Browser loopback UI and `compat-openclaw` exist but are not yet unified at the runtime layer | Browser and OpenClaw transport adapters reuse the same runtime service and session model |
| Gateway model | No explicit messenger-ingress model yet | Future IM or messenger gateways feed the same live runtime service without duplicating runtime logic |
| Validation | CLI smoke, HTTP smoke, and browser smoke exist separately | Fixture-backed test suite plus env-gated live smoke cover the unified runtime path |

## Scope Note

This plan is intentionally scoped to the missing live runtime slice. It does **not** re-plan the already-landed install/bootstrap/UI shell work, and it does **not** attempt full feature parity with FastClaw/OpenClaw in one step. It focuses on making the existing browser product path honest:

- one real runtime service
- one persisted session model
- one provider-backed live path
- one shared execution core across browser and OpenClaw transport adapters

Explicitly out of scope for this slice:

- implementing Slack/Telegram/Discord/Matrix gateways
- implementing agent-to-agent messenger routing
- introducing a separate “gateway layer” inside the runtime core

The internal runtime should stay singular; new channels should arrive later as additional adapters on top of it.

## Adapter and Gateway Design Notes

The implementation from this plan should preserve a generalized ingress model even before any IM gateway is built:

- **live runtime service**
  - accepts normalized runtime requests
  - emits normalized runtime events
  - has no knowledge of browser routes, OpenClaw frame names, or messenger APIs
- **transport adapters**
  - map one request/response protocol into the runtime request/event model
  - current scope:
    - browser loopback HTTP
    - OpenClaw-compatible HTTP/WebSocket
- **gateway adapters**
  - future ingress for IM and external channel systems
  - responsible for:
    - webhook or polling intake
    - sender/channel/thread normalization
    - outbound reply dispatch
    - delivery retries and channel-specific failure handling

The design bar for these external-facing layers is:

- as general as possible without becoming abstract nonsense
- strongly normalized at the runtime boundary
- explicit about identity, routing, and delivery semantics
- minimal messenger-specific branching inside shared code

## Execution Plan

```yaml
tasks:
  - id: "001-test"
    subject: "Write provider streaming adapter test"
    slug: "write-provider-streaming-adapter-test"
    type: "test"
    depends-on: []
  - id: "001-impl"
    subject: "Implement provider streaming adapter"
    slug: "implement-provider-streaming-adapter"
    type: "impl"
    depends-on: ["task-001-write-provider-streaming-adapter-test-test"]
  - id: "002-test"
    subject: "Write session-backed live run service test"
    slug: "write-session-backed-live-run-service-test"
    type: "test"
    depends-on: ["task-001-implement-provider-streaming-adapter-impl"]
  - id: "002-impl"
    subject: "Implement session-backed live run service"
    slug: "implement-session-backed-live-run-service"
    type: "impl"
    depends-on: ["task-002-write-session-backed-live-run-service-test-test"]
  - id: "003-test"
    subject: "Write workspace streaming transcript UI test"
    slug: "write-workspace-streaming-transcript-ui-test"
    type: "test"
    depends-on: ["task-002-implement-session-backed-live-run-service-impl"]
  - id: "003-impl"
    subject: "Implement workspace streaming transcript UI"
    slug: "implement-workspace-streaming-transcript-ui"
    type: "impl"
    depends-on: ["task-003-write-workspace-streaming-transcript-ui-test-test"]
  - id: "004-test"
    subject: "Write live queue integration test"
    slug: "write-live-queue-integration-test"
    type: "test"
    depends-on: ["task-002-implement-session-backed-live-run-service-impl"]
  - id: "004-impl"
    subject: "Implement live queue integration"
    slug: "implement-live-queue-integration"
    type: "impl"
    depends-on: ["task-004-write-live-queue-integration-test-test"]
  - id: "005-test"
    subject: "Write session resume over HTTP test"
    slug: "write-session-resume-over-http-test"
    type: "test"
    depends-on: ["task-002-implement-session-backed-live-run-service-impl"]
  - id: "005-impl"
    subject: "Implement session resume over HTTP"
    slug: "implement-session-resume-over-http"
    type: "impl"
    depends-on: ["task-005-write-session-resume-over-http-test-test"]
  - id: "006-test"
    subject: "Write live tool execution test"
    slug: "write-live-tool-execution-test"
    type: "test"
    depends-on: ["task-002-implement-session-backed-live-run-service-impl"]
  - id: "006-impl"
    subject: "Implement live tool execution"
    slug: "implement-live-tool-execution"
    type: "impl"
    depends-on: ["task-006-write-live-tool-execution-test-test"]
  - id: "007-test"
    subject: "Write blocked tool policy surfacing test"
    slug: "write-blocked-tool-policy-surfacing-test"
    type: "test"
    depends-on: ["task-006-implement-live-tool-execution-impl"]
  - id: "007-impl"
    subject: "Implement blocked tool policy surfacing"
    slug: "implement-blocked-tool-policy-surfacing"
    type: "impl"
    depends-on: ["task-007-write-blocked-tool-policy-surfacing-test-test"]
  - id: "008-test"
    subject: "Write compatibility runtime reuse test"
    slug: "write-compatibility-runtime-reuse-test"
    type: "test"
    depends-on: ["task-002-implement-session-backed-live-run-service-impl"]
  - id: "008-impl"
    subject: "Implement compatibility runtime reuse"
    slug: "implement-compatibility-runtime-reuse"
    type: "impl"
    depends-on: ["task-008-write-compatibility-runtime-reuse-test-test"]
  - id: "009-test"
    subject: "Write live smoke harness test"
    slug: "write-live-smoke-harness-test"
    type: "test"
    depends-on: ["task-003-implement-workspace-streaming-transcript-ui-impl", "task-008-implement-compatibility-runtime-reuse-impl"]
  - id: "009-impl"
    subject: "Implement live smoke harness"
    slug: "implement-live-smoke-harness"
    type: "impl"
    depends-on: ["task-009-write-live-smoke-harness-test-test"]
```

**Task File References (for detailed BDD scenarios):**
- [Task 001 Test: Write provider streaming adapter test](./task-001-write-provider-streaming-adapter-test-test.md)
- [Task 001 Impl: Implement provider streaming adapter](./task-001-implement-provider-streaming-adapter-impl.md)
- [Task 002 Test: Write session-backed live run service test](./task-002-write-session-backed-live-run-service-test-test.md)
- [Task 002 Impl: Implement session-backed live run service](./task-002-implement-session-backed-live-run-service-impl.md)
- [Task 003 Test: Write workspace streaming transcript UI test](./task-003-write-workspace-streaming-transcript-ui-test-test.md)
- [Task 003 Impl: Implement workspace streaming transcript UI](./task-003-implement-workspace-streaming-transcript-ui-impl.md)
- [Task 004 Test: Write live queue integration test](./task-004-write-live-queue-integration-test-test.md)
- [Task 004 Impl: Implement live queue integration](./task-004-implement-live-queue-integration-impl.md)
- [Task 005 Test: Write session resume over HTTP test](./task-005-write-session-resume-over-http-test-test.md)
- [Task 005 Impl: Implement session resume over HTTP](./task-005-implement-session-resume-over-http-impl.md)
- [Task 006 Test: Write live tool execution test](./task-006-write-live-tool-execution-test-test.md)
- [Task 006 Impl: Implement live tool execution](./task-006-implement-live-tool-execution-impl.md)
- [Task 007 Test: Write blocked tool policy surfacing test](./task-007-write-blocked-tool-policy-surfacing-test-test.md)
- [Task 007 Impl: Implement blocked tool policy surfacing](./task-007-implement-blocked-tool-policy-surfacing-impl.md)
- [Task 008 Test: Write compatibility runtime reuse test](./task-008-write-compatibility-runtime-reuse-test-test.md)
- [Task 008 Impl: Implement compatibility runtime reuse](./task-008-implement-compatibility-runtime-reuse-impl.md)
- [Task 009 Test: Write live smoke harness test](./task-009-write-live-smoke-harness-test-test.md)
- [Task 009 Impl: Implement live smoke harness](./task-009-implement-live-smoke-harness-impl.md)

## BDD Coverage

Scenarios already landed in prior batches and **not** re-planned here:
- `User installs MatrixClaw without privileged writes`
- `First launch opens setup without prior manual configuration`
- `User installs a text skill originally built for OpenClaw`
- `User installs a subprocess-compatible plugin originally built for OpenClaw`
- `User tries to install an in-process OpenClaw extension tied to JS internals`
- `Browser engine downloads only on first use`
- `Safe local execution works without Docker`
- `Optional sandbox mode is enabled explicitly`

Scenarios that this next-step plan completes end-to-end:
- `Final assistant answer is generated once` → task pair `001`
- `Stored transcript matches visible behavior` → task pairs `002`, `003`
- `Steering message is delivered before the next assistant turn` and `Follow-up message is delivered only after the current run completes` → task pair `004`
- `Session resumes after restart` → task pair `005`
- `Tool calls extend the turn loop` → task pair `006`
- `Tool validation can block unsafe execution` → task pair `007`
- `OpenClaw-compatible chat request reaches the internal runtime` → task pair `008`

Scope-specific live-product scenarios derived from the runtime model and current product gap:
- `Browser transcript streams deltas without duplicating the final assistant message` → task pair `003`
- `Live provider/browser smoke validates CLI, HTTP, and composer paths against one runtime` → task pair `009`

Additional architectural requirement carried by every task in this plan:
- browser and OpenClaw work must preserve a generalized adapter/gateway contract so future IM channels can be integrated without redesigning the live runtime service

## Dependency Chain

```text
task-001-test → task-001-impl
                      │
                      └─→ task-002-test → task-002-impl
                                             │
                                             ├─→ task-003-test → task-003-impl
                                             ├─→ task-004-test → task-004-impl
                                             ├─→ task-005-test → task-005-impl
                                             ├─→ task-006-test → task-006-impl
                                             │                      │
                                             │                      └─→ task-007-test → task-007-impl
                                             │
                                             └─→ task-008-test → task-008-impl

task-003-impl
      │
      └─→ task-009-test
task-008-impl
      │
      └─→ task-009-test → task-009-impl
```

**Analysis**:
- No circular dependencies are intended.
- The provider adapter must land before the unified live run service because every later slice depends on one real provider boundary.
- After the live run service lands, transcript UI, queue integration, session resume, live tool execution, and compatibility reuse can proceed as independent feature slices.
- Blocked-tool surfacing intentionally depends on live tool execution, because it is a safety branch of the same runtime path, not an independent feature.
- The smoke harness waits until both the browser transcript and compatibility/runtime reuse land, so final validation proves the shared runtime rather than only the browser shell.

---

## Execution Handoff

**Plan draft complete and saved to `docs/plans/2026-03-27-matrixclaw-live-agent-runtime-plan/`. Review options:**

**1. Plan Review First (Recommended)** - review this folder and approve any scope changes before commit.

**2. Orchestrated Execution After Approval** - use `superpowers:executing-plans`.

**3. Direct Focused Execution** - execute a specific pair such as `001` or `002` if you want to move incrementally.
