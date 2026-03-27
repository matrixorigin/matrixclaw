# Delivery Plan

## Purpose

This document turns the design package into an execution sequence.

The goal is to avoid the common failure mode where packaging, compatibility, and plugins are tackled too early and the runtime core never stabilizes.

## Delivery Principles

- ship the loop before the ecosystem
- ship the ecosystem before broad compatibility claims
- ship compatibility claims only with fixtures
- keep first-run success ahead of optional power features

## Phase 0: Design Lock

Objective:

- freeze the core runtime concepts before writing production code

Deliverables:

- architecture docs
- runtime model
- compatibility model
- schema drafts
- BDD scenarios

Exit criteria:

- message roles are defined
- queue semantics are defined
- support tiers are defined
- filesystem layout is defined

## Phase 1: Core Runtime Skeleton

Objective:

- create the Rust workspace and core crate boundaries

Suggested crates:

- `matrixclaw-agent-core`
- `matrixclaw-session-runtime`
- `matrixclaw-compat-openclaw`
- `matrixclaw-app-host`
- `matrixclaw-manifests`

Deliverables:

- core domain types
- event enums
- provider trait
- tool trait
- session storage interface
- config loader

Exit criteria:

- project compiles
- domain boundaries are encoded in crates
- no compatibility protocol types leak into core crates

## Phase 2: Streaming Loop And Persistence

Objective:

- make one end-to-end local conversation work correctly

Deliverables:

- streaming assistant message lifecycle
- finalized assistant persistence
- tool call and result persistence
- deterministic run/turn ids
- golden transcript tests

Exit criteria:

- one generation produces one final assistant message
- transcript matches streamed output
- tool results appear in persisted history

## Phase 3: Queueing, Retry, And Compaction

Objective:

- implement the runtime policies that make the product feel like an OpenClaw-class agent

Deliverables:

- steering queue
- follow-up queue
- retry classification and scheduling
- compaction records and summary insertion
- resume after restart

Exit criteria:

- steering and follow-up semantics match BDD specs
- overflow recovery works without corrupting history
- compaction never writes summary text as a user message

## Phase 4: Local Product Shell

Objective:

- make the binary usable without compatibility clients

Deliverables:

- CLI entrypoint
- first-run setup flow
- embedded web UI shell
- browser-first setup wizard
- minimal workspace explorer and file-reference insertion flow
- queued steering controls in the local chat surface
- config persistence
- loopback-local daemon mode

Exit criteria:

- clean install into user-owned path
- setup succeeds with no Bun, Node.js, or Docker requirement
- local prompt/send/stream flow works from UI and CLI
- the local UI already feels like a workspace tool, not just an admin page

## Phase 5: Skill And Workspace Compatibility

Objective:

- capture the easiest and highest-value community adoption path

Deliverables:

- `SKILL.md` import
- workspace file loading
- `matrixclaw.skill.json`
- compatibility inspection for skill artifacts
- global skill store plus agent-local enablement metadata
- `load_skill` runtime boundary for on-demand skill activation
- basic Skills management page and setup-wizard starter-skill selection

Exit criteria:

- common OpenClaw-style skills import unchanged
- provenance is recorded
- workspace convention precedence is documented and testable
- operators can see installed vs enabled skills without reading files directly

## Phase 6: Plugin Compatibility

Objective:

- support the plugin shapes that fit Rust cleanly

Deliverables:

- `matrixclaw.plugin.json`
- subprocess plugin launcher
- MCP integration path
- compatibility inspection for plugin artifacts
- precise unsupported diagnostics

Exit criteria:

- shimmed plugin installation works for at least one real artifact class
- unsupported in-process extensions fail clearly

## Phase 7: Protocol Compatibility

Objective:

- let OpenClaw-oriented clients talk to MatrixClaw

Deliverables:

- compatibility capability descriptor
- WebSocket connect flow
- `agents.list`
- `chat.start`
- `chat.stream`
- `chat.cancel`
- fixture-driven tests

Exit criteria:

- one real client can connect and chat successfully
- stream parity tests prove transcript and stream alignment

## Phase 8: Security Hardening And Operations

Objective:

- make the runtime safe enough to operate beyond a toy environment

Deliverables:

- asset verification
- secret redaction
- permission metadata enforcement where feasible
- observability and structured logs
- upgrade and backup commands

Exit criteria:

- managed assets are verified
- logs are redacted appropriately
- operational failure modes are documented and tested

## Deferred Work

These should stay out of the critical path unless the product needs them urgently:

- bridge runtime for JS or Bun plugins
- broad remote multi-user hosting
- advanced channel integrations
- heavyweight browser or media features
- rich embedded editor and document viewers
- authenticated web terminal
- registry service

## Test Gates By Phase

### Phase 2 gate

- golden transcript tests
- provider mock streaming tests

### Phase 3 gate

- queue semantics tests
- overflow and retry tests
- restart/resume tests

### Phase 5 gate

- skill import fixtures
- workspace precedence tests

### Phase 6 gate

- plugin classifier tests
- shim launch tests

### Phase 7 gate

- WebSocket fixture tests
- client interoperability smoke tests

## Recommended First Implementation Slice

If starting tomorrow, the smallest meaningful vertical slice is:

1. `agent-core` with streaming assistant lifecycle
2. `session-runtime` with SQLite-backed message persistence
3. one built-in local tool
4. one CLI chat flow
5. golden transcript tests

This gives a real product kernel before compatibility work starts.

## Design Checks

The delivery order is correct if:

- every later phase depends on a stable runtime core
- community compatibility starts with text skills before plugin bridges
- protocol support arrives only after stream and transcript correctness are proven
