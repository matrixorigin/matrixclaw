# MatrixClaw Runtime Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Load `superpowers:executing-plans` skill using the Skill tool to implement this plan task-by-task.

**Goal:** Build the first executable MatrixClaw runtime slice as a native Rust binary with a streaming-first agent loop, durable session runtime, OpenClaw compatibility boundaries, and installable ecosystem assets.

**Architecture:** The work is split into four crates and one supporting manifest crate: `matrixclaw-agent-core`, `matrixclaw-session-runtime`, `matrixclaw-compat-openclaw`, `matrixclaw-app-host`, and `matrixclaw-manifests`. Execution proceeds test-first from runtime kernel to session policy, then to compatibility, ecosystem adoption, and operator-facing install and asset flows.

**Tech Stack:** Rust stable, Cargo workspace, Tokio, Serde, Axum/WebSocket support, SQLite, test doubles for provider/tool/plugin boundaries, shell-based installer, and golden transcript / fixture-driven tests.

**Design Support:**
- [BDD Specs](../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md)
- [Architecture](../2026-03-26-rust-openclaw-runtime-design/architecture.md)
- [Runtime Model](../2026-03-26-rust-openclaw-runtime-design/runtime-model.md)
- [Protocol Compatibility](../2026-03-26-rust-openclaw-runtime-design/protocol-compatibility.md)
- [Ecosystem Compatibility](../2026-03-26-rust-openclaw-runtime-design/ecosystem-compatibility.md)
- [Schemas](../2026-03-26-rust-openclaw-runtime-design/schemas.md)
- [Install And Layout](../2026-03-26-rust-openclaw-runtime-design/install-and-layout.md)
- [Security And Operations](../2026-03-26-rust-openclaw-runtime-design/security-and-operations.md)
- [Delivery Plan](../2026-03-26-rust-openclaw-runtime-design/delivery-plan.md)

## Context

MatrixClaw currently has a strong design package and no implementation scaffold. The risk is not lack of ideas; it is allowing runtime policy, compatibility, and packaging concerns to collapse together during implementation and recreate the flaws already identified in FastClaw. This plan forces the work into small, test-first slices so the first shipped runtime preserves the architectural decisions already made: one generation per streamed answer, event-sourced persistence, compatibility at the boundary, and honest ecosystem support tiers.

Because this is effectively greenfield code inside an existing repo, the plan has to do two things simultaneously:

- create the Rust workspace and crate boundaries
- keep every feature slice tied to a BDD scenario with explicit Red/Green verification

| Aspect | Current State | Target State |
|--------|--------------|--------------|
| Repository shape | Design docs only | Cargo workspace with runtime crates, tests, and install paths |
| Agent loop | Specified in docs | Streaming-first `agent-core` with deterministic events |
| Session behavior | Specified in docs | Durable `session-runtime` with queue, retry, compaction, and resume |
| Compatibility | Scope defined only | Fixture-backed OpenClaw protocol adapter and ecosystem importer |
| Operator experience | No binary, no setup flow | User-owned install path, first-run setup, lazy assets, explicit sandbox policy, and a path toward skill/plugin management plus workspace-first UI |

## Scope Note

The design package now explicitly incorporates two upstream product lessons:

- from FastClaw
  - browser-first setup wizard, Skills/Plugins operator surfaces, and agent-local skill enablement
- from PiClaw
  - workspace-first chat UX, file-reference insertion, and queued steering controls in the UI

Those product learnings are important, but they do not change the current implementation ordering. The active task graph still prioritizes runtime correctness, persistence, compatibility boundaries, and basic local product shell behavior before richer operator UI and skill-management surfaces.

## Execution Plan

```yaml
tasks:
  - id: "001-test"
    subject: "Install without privileged writes test"
    slug: "install-without-privileged-writes-test"
    type: "test"
    depends-on: []
  - id: "001-impl"
    subject: "Install without privileged writes implementation"
    slug: "install-without-privileged-writes-impl"
    type: "impl"
    depends-on: ["task-001-install-without-privileged-writes-test"]
  - id: "002-test"
    subject: "First launch setup test"
    slug: "first-launch-setup-test"
    type: "test"
    depends-on: ["task-001-install-without-privileged-writes-impl"]
  - id: "002-impl"
    subject: "First launch setup implementation"
    slug: "first-launch-setup-impl"
    type: "impl"
    depends-on: ["task-002-first-launch-setup-test"]
  - id: "003-test"
    subject: "Single generation streaming test"
    slug: "single-generation-streaming-test"
    type: "test"
    depends-on: ["task-001-install-without-privileged-writes-impl"]
  - id: "003-impl"
    subject: "Single generation streaming implementation"
    slug: "single-generation-streaming-impl"
    type: "impl"
    depends-on: ["task-003-single-generation-streaming-test"]
  - id: "004-test"
    subject: "Tool calls extend loop test"
    slug: "tool-calls-extend-loop-test"
    type: "test"
    depends-on: ["task-003-single-generation-streaming-impl"]
  - id: "004-impl"
    subject: "Tool calls extend loop implementation"
    slug: "tool-calls-extend-loop-impl"
    type: "impl"
    depends-on: ["task-004-tool-calls-extend-loop-test"]
  - id: "005-test"
    subject: "Tool preflight block test"
    slug: "tool-preflight-block-test"
    type: "test"
    depends-on: ["task-004-tool-calls-extend-loop-impl"]
  - id: "005-impl"
    subject: "Tool preflight block implementation"
    slug: "tool-preflight-block-impl"
    type: "impl"
    depends-on: ["task-005-tool-preflight-block-test"]
  - id: "006-test"
    subject: "Steering queue delivery test"
    slug: "steering-queue-delivery-test"
    type: "test"
    depends-on: ["task-004-tool-calls-extend-loop-impl"]
  - id: "006-impl"
    subject: "Steering queue delivery implementation"
    slug: "steering-queue-delivery-impl"
    type: "impl"
    depends-on: ["task-006-steering-queue-delivery-test"]
  - id: "007-test"
    subject: "Follow-up queue delivery test"
    slug: "follow-up-queue-delivery-test"
    type: "test"
    depends-on: ["task-006-steering-queue-delivery-impl"]
  - id: "007-impl"
    subject: "Follow-up queue delivery implementation"
    slug: "follow-up-queue-delivery-impl"
    type: "impl"
    depends-on: ["task-007-follow-up-queue-delivery-test"]
  - id: "008-test"
    subject: "Transcript parity test"
    slug: "transcript-parity-test"
    type: "test"
    depends-on: ["task-005-tool-preflight-block-impl"]
  - id: "008-impl"
    subject: "Transcript parity implementation"
    slug: "transcript-parity-impl"
    type: "impl"
    depends-on: ["task-008-transcript-parity-test"]
  - id: "009-test"
    subject: "Session resume after restart test"
    slug: "session-resume-after-restart-test"
    type: "test"
    depends-on: ["task-008-transcript-parity-impl"]
  - id: "009-impl"
    subject: "Session resume after restart implementation"
    slug: "session-resume-after-restart-impl"
    type: "impl"
    depends-on: ["task-009-session-resume-after-restart-test"]
  - id: "010-test"
    subject: "Overflow compaction retry test"
    slug: "overflow-compaction-retry-test"
    type: "test"
    depends-on: ["task-008-transcript-parity-impl"]
  - id: "010-impl"
    subject: "Overflow compaction retry implementation"
    slug: "overflow-compaction-retry-impl"
    type: "impl"
    depends-on: ["task-010-overflow-compaction-retry-test"]
  - id: "011-test"
    subject: "Compaction role semantics test"
    slug: "compaction-role-semantics-test"
    type: "test"
    depends-on: ["task-010-overflow-compaction-retry-impl"]
  - id: "011-impl"
    subject: "Compaction role semantics implementation"
    slug: "compaction-role-semantics-impl"
    type: "impl"
    depends-on: ["task-011-compaction-role-semantics-test"]
  - id: "012-test"
    subject: "OpenClaw agents list test"
    slug: "openclaw-agents-list-test"
    type: "test"
    depends-on: ["task-002-first-launch-setup-impl"]
  - id: "012-impl"
    subject: "OpenClaw agents list implementation"
    slug: "openclaw-agents-list-impl"
    type: "impl"
    depends-on: ["task-012-openclaw-agents-list-test"]
  - id: "013-test"
    subject: "OpenClaw chat translation test"
    slug: "openclaw-chat-translation-test"
    type: "test"
    depends-on: ["task-008-transcript-parity-impl", "task-012-openclaw-agents-list-impl"]
  - id: "013-impl"
    subject: "OpenClaw chat translation implementation"
    slug: "openclaw-chat-translation-impl"
    type: "impl"
    depends-on: ["task-013-openclaw-chat-translation-test"]
  - id: "014-test"
    subject: "OpenClaw text skill import test"
    slug: "openclaw-text-skill-import-test"
    type: "test"
    depends-on: ["task-001-install-without-privileged-writes-impl"]
  - id: "014-impl"
    subject: "OpenClaw text skill import implementation"
    slug: "openclaw-text-skill-import-impl"
    type: "impl"
    depends-on: ["task-014-openclaw-text-skill-import-test"]
  - id: "015-test"
    subject: "OpenClaw subprocess plugin install test"
    slug: "openclaw-subprocess-plugin-install-test"
    type: "test"
    depends-on: ["task-014-openclaw-text-skill-import-impl"]
  - id: "015-impl"
    subject: "OpenClaw subprocess plugin install implementation"
    slug: "openclaw-subprocess-plugin-install-impl"
    type: "impl"
    depends-on: ["task-015-openclaw-subprocess-plugin-install-test"]
  - id: "016-test"
    subject: "OpenClaw in-process extension rejection test"
    slug: "openclaw-inprocess-extension-rejection-test"
    type: "test"
    depends-on: ["task-015-openclaw-subprocess-plugin-install-impl"]
  - id: "016-impl"
    subject: "OpenClaw in-process extension rejection implementation"
    slug: "openclaw-inprocess-extension-rejection-impl"
    type: "impl"
    depends-on: ["task-016-openclaw-inprocess-extension-rejection-test"]
  - id: "017-test"
    subject: "Browser asset lazy download test"
    slug: "browser-asset-lazy-download-test"
    type: "test"
    depends-on: ["task-002-first-launch-setup-impl"]
  - id: "017-impl"
    subject: "Browser asset lazy download implementation"
    slug: "browser-asset-lazy-download-impl"
    type: "impl"
    depends-on: ["task-017-browser-asset-lazy-download-test"]
  - id: "018-test"
    subject: "Local execution without Docker test"
    slug: "local-execution-without-docker-test"
    type: "test"
    depends-on: ["task-005-tool-preflight-block-impl"]
  - id: "018-impl"
    subject: "Local execution without Docker implementation"
    slug: "local-execution-without-docker-impl"
    type: "impl"
    depends-on: ["task-018-local-execution-without-docker-test"]
  - id: "019-test"
    subject: "Optional sandbox mode test"
    slug: "optional-sandbox-mode-test"
    type: "test"
    depends-on: ["task-018-local-execution-without-docker-impl"]
  - id: "019-impl"
    subject: "Optional sandbox mode implementation"
    slug: "optional-sandbox-mode-impl"
    type: "impl"
    depends-on: ["task-019-optional-sandbox-mode-test"]
```

**Task File References (for detailed BDD scenarios):**
- [Task 001 Test: Install without privileged writes](./task-001-install-without-privileged-writes-test.md)
- [Task 001 Impl: Install without privileged writes](./task-001-install-without-privileged-writes-impl.md)
- [Task 002 Test: First launch setup](./task-002-first-launch-setup-test.md)
- [Task 002 Impl: First launch setup](./task-002-first-launch-setup-impl.md)
- [Task 003 Test: Single generation streaming](./task-003-single-generation-streaming-test.md)
- [Task 003 Impl: Single generation streaming](./task-003-single-generation-streaming-impl.md)
- [Task 004 Test: Tool calls extend loop](./task-004-tool-calls-extend-loop-test.md)
- [Task 004 Impl: Tool calls extend loop](./task-004-tool-calls-extend-loop-impl.md)
- [Task 005 Test: Tool preflight block](./task-005-tool-preflight-block-test.md)
- [Task 005 Impl: Tool preflight block](./task-005-tool-preflight-block-impl.md)
- [Task 006 Test: Steering queue delivery](./task-006-steering-queue-delivery-test.md)
- [Task 006 Impl: Steering queue delivery](./task-006-steering-queue-delivery-impl.md)
- [Task 007 Test: Follow-up queue delivery](./task-007-follow-up-queue-delivery-test.md)
- [Task 007 Impl: Follow-up queue delivery](./task-007-follow-up-queue-delivery-impl.md)
- [Task 008 Test: Transcript parity](./task-008-transcript-parity-test.md)
- [Task 008 Impl: Transcript parity](./task-008-transcript-parity-impl.md)
- [Task 009 Test: Session resume after restart](./task-009-session-resume-after-restart-test.md)
- [Task 009 Impl: Session resume after restart](./task-009-session-resume-after-restart-impl.md)
- [Task 010 Test: Overflow compaction retry](./task-010-overflow-compaction-retry-test.md)
- [Task 010 Impl: Overflow compaction retry](./task-010-overflow-compaction-retry-impl.md)
- [Task 011 Test: Compaction role semantics](./task-011-compaction-role-semantics-test.md)
- [Task 011 Impl: Compaction role semantics](./task-011-compaction-role-semantics-impl.md)
- [Task 012 Test: OpenClaw agents list](./task-012-openclaw-agents-list-test.md)
- [Task 012 Impl: OpenClaw agents list](./task-012-openclaw-agents-list-impl.md)
- [Task 013 Test: OpenClaw chat translation](./task-013-openclaw-chat-translation-test.md)
- [Task 013 Impl: OpenClaw chat translation](./task-013-openclaw-chat-translation-impl.md)
- [Task 014 Test: OpenClaw text skill import](./task-014-openclaw-text-skill-import-test.md)
- [Task 014 Impl: OpenClaw text skill import](./task-014-openclaw-text-skill-import-impl.md)
- [Task 015 Test: OpenClaw subprocess plugin install](./task-015-openclaw-subprocess-plugin-install-test.md)
- [Task 015 Impl: OpenClaw subprocess plugin install](./task-015-openclaw-subprocess-plugin-install-impl.md)
- [Task 016 Test: OpenClaw in-process extension rejection](./task-016-openclaw-inprocess-extension-rejection-test.md)
- [Task 016 Impl: OpenClaw in-process extension rejection](./task-016-openclaw-inprocess-extension-rejection-impl.md)
- [Task 017 Test: Browser asset lazy download](./task-017-browser-asset-lazy-download-test.md)
- [Task 017 Impl: Browser asset lazy download](./task-017-browser-asset-lazy-download-impl.md)
- [Task 018 Test: Local execution without Docker](./task-018-local-execution-without-docker-test.md)
- [Task 018 Impl: Local execution without Docker](./task-018-local-execution-without-docker-impl.md)
- [Task 019 Test: Optional sandbox mode](./task-019-optional-sandbox-mode-test.md)
- [Task 019 Impl: Optional sandbox mode](./task-019-optional-sandbox-mode-impl.md)

## BDD Coverage

| BDD Scenario | Red Task | Green Task |
|---|---|---|
| User installs MatrixClaw without privileged writes | `task-001-install-without-privileged-writes-test` | `task-001-install-without-privileged-writes-impl` |
| First launch opens setup without prior manual configuration | `task-002-first-launch-setup-test` | `task-002-first-launch-setup-impl` |
| Final assistant answer is generated once | `task-003-single-generation-streaming-test` | `task-003-single-generation-streaming-impl` |
| Tool calls extend the turn loop | `task-004-tool-calls-extend-loop-test` | `task-004-tool-calls-extend-loop-impl` |
| Tool validation can block unsafe execution | `task-005-tool-preflight-block-test` | `task-005-tool-preflight-block-impl` |
| Steering message is delivered before the next assistant turn | `task-006-steering-queue-delivery-test` | `task-006-steering-queue-delivery-impl` |
| Follow-up message is delivered only after the current run completes | `task-007-follow-up-queue-delivery-test` | `task-007-follow-up-queue-delivery-impl` |
| Stored transcript matches visible behavior | `task-008-transcript-parity-test` | `task-008-transcript-parity-impl` |
| Session resumes after restart | `task-009-session-resume-after-restart-test` | `task-009-session-resume-after-restart-impl` |
| Runtime compacts context before retrying an overflowed request | `task-010-overflow-compaction-retry-test` | `task-010-overflow-compaction-retry-impl` |
| Compaction preserves role semantics | `task-011-compaction-role-semantics-test` | `task-011-compaction-role-semantics-impl` |
| OpenClaw-compatible client lists agents | `task-012-openclaw-agents-list-test` | `task-012-openclaw-agents-list-impl` |
| OpenClaw-compatible chat request reaches the internal runtime | `task-013-openclaw-chat-translation-test` | `task-013-openclaw-chat-translation-impl` |
| User installs a text skill originally built for OpenClaw | `task-014-openclaw-text-skill-import-test` | `task-014-openclaw-text-skill-import-impl` |
| User installs a subprocess-compatible plugin originally built for OpenClaw | `task-015-openclaw-subprocess-plugin-install-test` | `task-015-openclaw-subprocess-plugin-install-impl` |
| User tries to install an in-process OpenClaw extension tied to JS internals | `task-016-openclaw-inprocess-extension-rejection-test` | `task-016-openclaw-inprocess-extension-rejection-impl` |
| Browser engine downloads only on first use | `task-017-browser-asset-lazy-download-test` | `task-017-browser-asset-lazy-download-impl` |
| Safe local execution works without Docker | `task-018-local-execution-without-docker-test` | `task-018-local-execution-without-docker-impl` |
| Optional sandbox mode is enabled explicitly | `task-019-optional-sandbox-mode-test` | `task-019-optional-sandbox-mode-impl` |

## Dependency Chain

```text
task-001
  ├─→ task-002 ──┬─→ task-012 ──→ task-013
  │              └─→ task-017
  ├─→ task-003 ──→ task-004 ──┬─→ task-005 ──┬─→ task-008 ──┬─→ task-009
  │                            │              │              └─→ task-010 ──→ task-011
  │                            │              └─→ task-018 ──→ task-019
  │                            └─→ task-006 ──→ task-007
  └─→ task-014 ──→ task-015 ──→ task-016
```

**Analysis**:
- No circular dependencies are planned.
- The graph keeps three major parallel tracks available after the workspace exists: runtime core, operator setup/assets, and ecosystem import work.
- Compatibility chat translation waits for both session transcript parity and the agent-list handshake so the adapter is built on stable runtime behavior rather than speculative protocol glue.

---

## Execution Handoff

**Plan folder:** `docs/plans/2026-03-26-rust-openclaw-runtime-plan/`

**Execution options:**

**1. Orchestrated Execution (Recommended)** - Load `superpowers:executing-plans` to run the task graph with dependency awareness.

**2. Direct Agent Team** - Load `superpowers:agent-team-driven-development` for a coordinated multi-worker implementation pass.

**3. BDD-Focused Execution** - Load `superpowers:behavior-driven-development` if you want to drive implementation scenario-by-scenario.
