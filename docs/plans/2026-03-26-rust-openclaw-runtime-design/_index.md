# Rust OpenClaw Runtime Design

## Context

The target product is a self-hosted AI agent runtime that preserves OpenClaw-class capabilities while being installable as a native binary without requiring Bun, Node.js, Docker, or other mandatory runtime dependencies during initial terminal setup.

Three upstream references shape the design:

- `badlogic/pi-mono`
  - best reference for the core agent loop, event model, queued message handling, tool execution lifecycle, and separation of loop concerns from session/runtime concerns
- `fastclaw-ai/fastclaw`
  - best reference for product packaging goals: single binary, embedded setup flow, embedded web UI, explicit Skills/Plugins operator surfaces, and self-hosted runtime ergonomics
- OpenClaw ecosystem
  - compatibility target for protocol, client integration, and operator expectations

`piclaw` is intentionally not the architectural reference. It is useful for workspace-first product ideas such as file reference UX, queued steering controls, and a richer single-user web surface, but its Bun-first and container-first runtime model is not aligned with the desired binary-first Rust product.

## Requirements

### Functional requirements

- Ship as a native Rust binary installable by a simple shell command or downloaded release artifact
- Run as a self-hosted agent runtime with web UI and CLI/TUI control surfaces
- Preserve OpenClaw-like behavior:
  - multi-turn tool-using agent loop
  - persistent sessions
  - queued steering and follow-up messages
  - session compaction and retry handling
  - agent workspaces with long-lived memory files and searchable history
- Provide explicit OpenClaw compatibility in two areas:
  - client/protocol compatibility for existing clients and integrations
  - ecosystem compatibility for installable skills and selected plugin types
- Support a provider abstraction with OpenAI-compatible APIs first, without hard-coding the loop to a single vendor
- Support pluggable tools, including subprocess tools and future MCP tools

### Distribution requirements

- No Bun/Node/Docker requirement for basic installation or first run
- No required `sudo` path writes during install by default
- Embedded setup experience on first launch
- Embedded static web assets in the binary
- Optional heavyweight components handled as managed lazy downloads, not required install-time dependencies
- Operator-visible skill and plugin management surfaces in the web UI
- Workspace-first chat UX with file reference and queued steering controls

### Quality requirements

- Streaming-first execution model
- Strong loop/session/runtime separation
- Deterministic event model suitable for web UI, TUI, automation, and testing
- Persistent transcripts that exactly match user-visible behavior
- Compatibility layer treated as a boundary, not as the internal architecture

## Non-Goals

- Perfect line-by-line behavioral compatibility with FastClaw internals
- Reproducing `piclaw`'s container environment or Bun runtime
- Shipping every optional channel and viewer in v1
- Building the entire OpenClaw ecosystem before the core loop and session runtime are stable

## Recommended Direction

Build MatrixClaw as four layers:

1. `agent-core`
   - pure Rust loop engine modeled after `pi-agent-core`
2. `session-runtime`
   - persistence, compaction, retry, steering/follow-up queues, model/session policy
3. `compat-openclaw`
   - WebSocket/HTTP client shims plus ecosystem import/adaptation helpers
4. `app-host`
   - binary distribution, setup UI, embedded web UI, local daemon mode, channels, managed downloads

This keeps the internal architecture clean while still meeting compatibility and packaging goals.

## Rationale

### Why learn the loop from `pi-mono`

`pi-agent-core` has the right boundary discipline:

- the loop works on rich agent messages, not provider wire messages
- context transformation is separate from provider conversion
- streaming is the primary execution path
- tool execution has an explicit lifecycle with preflight and post-processing hooks
- queued steering and follow-up messages are modeled directly in the loop

That architecture is a better base for Rust than FastClaw's more product-coupled loop.

### Why learn packaging from FastClaw

FastClaw is much closer to the product goal:

- single-binary mental model
- embedded setup flow
- embedded web UI
- explicit operator surfaces for agents, skills, plugins, and settings
- agent-local organization for memory, personality, and skills
- on-demand `load_skill` behavior instead of assuming all skills are always active
- self-hosted gateway
- OpenClaw-facing API posture

Its product instincts are right even where its loop design is weaker.

### What to learn from PiClaw UI

PiClaw is useful as a product-surface reference even though it is not the runtime reference:

- workspace explorer and file preview are better than a chat-only shell
- attaching file references into prompts is a high-leverage operator workflow
- steering and follow-up controls belong in the chat UI, not only in protocol docs
- a single-user mobile-friendly web surface is a better default than an admin dashboard alone

MatrixClaw should copy those UX instincts without copying PiClaw's Bun runtime, container assumptions, or heavy first-run dependencies.

### Why compatibility must be a shim

OpenClaw compatibility is important, but it should be implemented as an edge contract:

- protocol adapters
- session import/export
- compatibility headers and payloads
- ecosystem import/adaptation for compatible skills and plugins

The internal runtime should not be forced into a less coherent design just to mirror an external protocol.

## Compatibility Scope

MatrixClaw must distinguish two different compatibility goals.

### 1. Client/protocol compatibility

This covers interoperability with OpenClaw-oriented clients and tooling:

- WebSocket protocol
- HTTP API shape
- auth headers/tokens
- agent listing and chat request framing
- session import/export where needed

### 2. Ecosystem compatibility

This covers whether users can install artifacts originally built for OpenClaw:

- skills
- prompt bundles
- workspace conventions
- plugin packages

These should not be treated as the same problem.

## Ecosystem Compatibility Tiers

### Tier 1: native support

MatrixClaw should strongly support OpenClaw-style artifacts that are mostly data or prompts:

- markdown/text skills
- prompt templates
- workspace files like `AGENTS.md`, `SOUL.md`, `MEMORY.md`, `USER.md`, `TOOLS.md`
- declarative metadata files that do not depend on JS runtime internals

### Tier 2: shimmed support

MatrixClaw should support plugin types that can be adapted through stable process boundaries:

- subprocess plugins
- JSON-RPC plugins
- MCP servers and MCP-style tool adapters

These should run through explicit adapter layers or bridges.

### Tier 3: unsupported or bridged-only

MatrixClaw should not promise native compatibility for artifacts that depend on OpenClaw or `pi` internal runtime APIs:

- in-process TypeScript/Bun extensions
- packages that expect exact OpenClaw internal event objects
- packages that depend on Node/Bun module resolution or monkey-patching runtime internals

If support is ever added here, it should be through an optional bridge runtime, not the core Rust architecture.

## Detailed Design Summary

### Core idea

The runtime uses a streaming-first turn engine that emits structured events. Those events are consumed by the session runtime for persistence and by surfaces like web UI, TUI, and compatibility APIs.

### Operating model

- User input or external channel message enters `session-runtime`
- Runtime appends a new user/app message to session state
- `agent-core` runs one or more turns
- Assistant stream events are emitted incrementally
- Tool calls are validated, executed, and reported through structured events
- Tool result messages are persisted as first-class messages
- If the assistant finishes without more tool calls, the run ends
- If steering or follow-up messages are queued, the loop continues according to delivery policy

### Install and first-run experience

- installer places the binary in a user-owned path such as `~/.matrixclaw/bin`
- first launch starts a local setup server or TUI wizard
- setup writes config and initializes workspace/session directories
- setup should expose provider selection, agent bootstrap, and initial skill enablement in one operator flow
- optional browser engine, OCR/STT models, or other large assets download only when first needed

### Skill and workspace operator model

- skills install into a global MatrixClaw skill store
- agents enable a selected subset of installed skills
- the runtime can load enabled skills on demand rather than forcing eager activation
- the web UI should expose dedicated Skills and Plugins views plus agent-local enablement controls
- the chat/workspace UI should support file reference insertion, queued steering, and follow-up controls

### Initial v1 scope

- native binary
- local web UI
- local CLI/TUI
- OpenAI-compatible provider support
- file/SQLite-backed sessions
- tool execution with local and sandboxed modes
- OpenClaw-compatible WebSocket/HTTP boundary sufficient for client interoperability
- native support for OpenClaw-style text skills and workspace conventions
- shim path for subprocess or MCP-style plugin compatibility
- browser-first setup wizard and basic Skills/Plugins management views
- workspace explorer with file-reference insertion into prompts

### Deferred product scope after the first runtime slice

- rich embedded editor and heavyweight viewers
- authenticated web terminal
- theme/tint customization and other polish features
- advanced channel administration

## Risks

- OpenClaw protocol drift if compatibility is implemented without fixture-based tests
- overreaching into too many channels before loop/session correctness is stable
- mixing compaction and persistence responsibilities back into the loop layer
- allowing install-time simplicity to be undermined by hidden first-run prerequisites

## Success Criteria

- A user can install the binary and launch the runtime without separately installing Bun, Node.js, or Docker
- The same assistant response is streamed, persisted, and exposed to clients without double-generation
- Tool execution results, failures, and queued user messages are reflected consistently across session storage and UIs
- OpenClaw-oriented clients can connect through a compatibility boundary without forcing internal architectural compromises
- A user can install common OpenClaw-style skills without rewriting them for MatrixClaw
- MatrixClaw clearly documents which OpenClaw plugin types are natively supported, shimmed, or unsupported

## Design Documents

- [BDD Specifications](./bdd-specs.md) - Behavior scenarios and testing strategy
- [Architecture](./architecture.md) - System architecture and component details
- [Runtime Model](./runtime-model.md) - State model, event flow, turn lifecycle, retry and compaction semantics
- [Protocol Compatibility](./protocol-compatibility.md) - OpenClaw-facing protocol boundary, capability versioning, and fixture strategy
- [Best Practices](./best-practices.md) - Security, performance, and code quality guidelines
- [Ecosystem Compatibility](./ecosystem-compatibility.md) - OpenClaw skill/plugin adoption model and compatibility tiers
- [Schemas](./schemas.md) - Proposed manifest formats, inspect output, and provenance records
- [Install And Layout](./install-and-layout.md) - Filesystem layout, install/import flow, and lifecycle commands
- [Security And Operations](./security-and-operations.md) - Trust boundaries, sandbox policy, asset verification, upgrades, and observability
- [Migration Guide](./migration-guide.md) - How existing OpenClaw assets map into MatrixClaw
- [Delivery Plan](./delivery-plan.md) - Phased implementation order, milestones, and exit criteria

## Recommended Reading Order

For architecture review:

1. [Architecture](./architecture.md)
2. [Runtime Model](./runtime-model.md)
3. [Protocol Compatibility](./protocol-compatibility.md)
4. [Ecosystem Compatibility](./ecosystem-compatibility.md)
5. [Security And Operations](./security-and-operations.md)

For implementation planning:

1. [BDD Specifications](./bdd-specs.md)
2. [Schemas](./schemas.md)
3. [Install And Layout](./install-and-layout.md)
4. [Migration Guide](./migration-guide.md)
5. [Delivery Plan](./delivery-plan.md)
