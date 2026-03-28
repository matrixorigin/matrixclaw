# MatrixClaw Served Transport Adapter Plan

**Goal:** Finish the next product-critical slice by serving the shared live runtime through real OpenClaw-compatible HTTP and WebSocket transport adapters, while introducing a normalized ingress contract that future IM gateways can reuse without changing runtime semantics.

**Architecture:** Keep `matrixclaw-app-host` as the product host for all served transports. `matrixclaw-compat-openclaw` should remain protocol-shaped and own frame/auth/capability translation only. `app-host` should host real OpenClaw HTTP and WebSocket endpoints that delegate into the same `SessionBackedLiveRunService` used by the browser SSE path. Add a normalized inbound envelope contract between served transports and the live runtime so future Slack/Telegram/Discord/Matrix gateways can reuse the same execution core.

**Tech Stack:** Rust stable, Cargo workspace, `tiny_http` loopback server, `matrixclaw-app-host`, `matrixclaw-compat-openclaw`, `matrixclaw-session-runtime`, `matrixclaw-agent-core`, `reqwest` for HTTP verification, browser SSE transport, provider-backed smoke via OpenRouter/Kimi.

**Design Support:**
- [Live Runtime Plan](../2026-03-27-matrixclaw-live-agent-runtime-plan/_index.md)
- [Architecture](../2026-03-26-rust-openclaw-runtime-design/architecture.md)
- [Runtime Model](../2026-03-26-rust-openclaw-runtime-design/runtime-model.md)
- [Protocol Compatibility](../2026-03-26-rust-openclaw-runtime-design/protocol-compatibility.md)

## Context

MatrixClaw now has one real live runtime service, one session store, one browser streaming path, and one internal OpenClaw transport adapter. That is the right internal architecture, but it is still not exposed as a served external transport boundary. OpenClaw compatibility currently exists as a library call inside `app-host`, not as a loopback HTTP/WebSocket surface clients can actually connect to.

That leaves the product in an in-between state:

- the runtime is shared
- the compatibility boundary is partially real
- the served transport layer is still missing

This next phase should complete that boundary honestly. It should not add more runtime features. It should expose the runtime through served transport adapters and lock in a gateway-ready ingress contract so future messenger integrations do not reshape the core.

| Aspect | Current State | Target State |
|--------|--------------|--------------|
| Browser transport | Real loopback HTTP + SSE served by `app-host` | Unchanged, remains one served transport adapter |
| OpenClaw HTTP | Internal adapter only | Real served OpenClaw HTTP endpoint in `app-host` |
| OpenClaw WebSocket | Protocol helpers only | Real served OpenClaw WebSocket conversation handling in `app-host` |
| Transport ownership | Split between internal helpers and host | `app-host` owns served transports, `compat-openclaw` owns protocol translation only |
| Runtime reuse | Shared internally | Proven across browser, OpenClaw HTTP, and OpenClaw WebSocket |
| Streaming parity | Browser streams, OpenClaw path buffers | Both transports project live runtime events progressively |
| Gateway readiness | Architectural intention only | Explicit normalized ingress envelope and adapter contract |
| Validation | Browser smoke and internal adapter tests | Cross-transport smoke proving one persisted session model |

## Scope Note

This phase is intentionally limited to served transport adapters and ingress normalization.

In scope:
- served OpenClaw-compatible HTTP endpoint in `app-host`
- served OpenClaw-compatible WebSocket handling in `app-host`
- progressive event projection for OpenClaw transport
- normalized ingress envelope for future gateways
- cross-transport session reuse validation

Out of scope:
- new agent features
- new browser UI pages
- actual Slack/Telegram/Discord/Matrix connectors
- multi-user auth model redesign
- desktop shell or Tauri work

## Execution Plan

```yaml
tasks:
  - id: "001-test"
    subject: "Write served OpenClaw HTTP transport test"
    slug: "write-served-openclaw-http-transport-test"
    type: "test"
    depends-on: []
  - id: "001-impl"
    subject: "Implement served OpenClaw HTTP transport"
    slug: "implement-served-openclaw-http-transport"
    type: "impl"
    depends-on: ["task-001-write-served-openclaw-http-transport-test-test"]
  - id: "002-test"
    subject: "Write served OpenClaw WebSocket transport test"
    slug: "write-served-openclaw-websocket-transport-test"
    type: "test"
    depends-on: ["task-001-implement-served-openclaw-http-transport-impl"]
  - id: "002-impl"
    subject: "Implement served OpenClaw WebSocket transport"
    slug: "implement-served-openclaw-websocket-transport"
    type: "impl"
    depends-on: ["task-002-write-served-openclaw-websocket-transport-test-test"]
  - id: "003-test"
    subject: "Write OpenClaw streaming parity test"
    slug: "write-openclaw-streaming-parity-test"
    type: "test"
    depends-on: ["task-001-implement-served-openclaw-http-transport-impl", "task-002-implement-served-openclaw-websocket-transport-impl"]
  - id: "003-impl"
    subject: "Implement OpenClaw streaming parity"
    slug: "implement-openclaw-streaming-parity"
    type: "impl"
    depends-on: ["task-003-write-openclaw-streaming-parity-test-test"]
  - id: "004-test"
    subject: "Write normalized ingress envelope contract test"
    slug: "write-normalized-ingress-envelope-contract-test"
    type: "test"
    depends-on: ["task-001-implement-served-openclaw-http-transport-impl"]
  - id: "004-impl"
    subject: "Implement normalized ingress envelope contract"
    slug: "implement-normalized-ingress-envelope-contract"
    type: "impl"
    depends-on: ["task-004-write-normalized-ingress-envelope-contract-test-test"]
  - id: "005-test"
    subject: "Write cross-transport session reuse smoke test"
    slug: "write-cross-transport-session-reuse-smoke-test"
    type: "test"
    depends-on: ["task-003-implement-openclaw-streaming-parity-impl", "task-004-implement-normalized-ingress-envelope-contract-impl"]
  - id: "005-impl"
    subject: "Implement cross-transport session reuse smoke harness"
    slug: "implement-cross-transport-session-reuse-smoke-harness"
    type: "impl"
    depends-on: ["task-005-write-cross-transport-session-reuse-smoke-test-test"]
```

## Task File References

- [Task 001 Test: Write served OpenClaw HTTP transport test](./task-001-write-served-openclaw-http-transport-test-test.md)
- [Task 001 Impl: Implement served OpenClaw HTTP transport](./task-001-implement-served-openclaw-http-transport-impl.md)
- [Task 002 Test: Write served OpenClaw WebSocket transport test](./task-002-write-served-openclaw-websocket-transport-test-test.md)
- [Task 002 Impl: Implement served OpenClaw WebSocket transport](./task-002-implement-served-openclaw-websocket-transport-impl.md)
- [Task 003 Test: Write OpenClaw streaming parity test](./task-003-write-openclaw-streaming-parity-test-test.md)
- [Task 003 Impl: Implement OpenClaw streaming parity](./task-003-implement-openclaw-streaming-parity-impl.md)
- [Task 004 Test: Write normalized ingress envelope contract test](./task-004-write-normalized-ingress-envelope-contract-test-test.md)
- [Task 004 Impl: Implement normalized ingress envelope contract](./task-004-implement-normalized-ingress-envelope-contract-impl.md)
- [Task 005 Test: Write cross-transport session reuse smoke test](./task-005-write-cross-transport-session-reuse-smoke-test-test.md)
- [Task 005 Impl: Implement cross-transport session reuse smoke harness](./task-005-implement-cross-transport-session-reuse-smoke-harness-impl.md)

## BDD Coverage

This phase covers the next unshipped transport scenarios:

- `OpenClaw HTTP client reaches the shared live runtime through a served endpoint` → task pair `001`
- `OpenClaw WebSocket client reaches the shared live runtime through a served conversation transport` → task pair `002`
- `OpenClaw transport streams runtime events progressively without diverging from browser semantics` → task pair `003`
- `External channel input is normalized before entering the live runtime core` → task pair `004`
- `Browser and OpenClaw transports can resume the same persisted session` → task pair `005`

Architectural rule carried by every task in this plan:

- browser, OpenClaw, and future IM gateways remain transport adapters over one live runtime service
- messenger- or protocol-specific naming stays outside the runtime core

## Dependency Chain

```text
task-001-test → task-001-impl
                      │
                      ├─→ task-002-test → task-002-impl
                      └─→ task-004-test → task-004-impl

task-001-impl
      │
task-002-impl
      └─→ task-003-test → task-003-impl

task-003-impl
      │
task-004-impl
      └─→ task-005-test → task-005-impl
```

**Analysis**:
- HTTP serving lands first because both WebSocket handling and ingress normalization need a real served transport home in `app-host`.
- WebSocket handling and ingress normalization can proceed independently once HTTP transport ownership is established.
- Streaming parity waits until both served OpenClaw transports exist.
- Cross-transport smoke waits until event streaming and normalized ingress contracts are both stable, so the final harness proves the real transport shape rather than a partially wired boundary.

## Execution Handoff

**Plan draft complete and saved to `docs/plans/2026-03-28-matrixclaw-served-transport-plan/`. Review options:**

**1. Plan Review First (Recommended)** - review this folder and tighten any transport/API expectations before commit.

**2. Orchestrated Execution After Approval** - use `superpowers:executing-plans`.

**3. Focused First Slice** - execute task pair `001` first if you want the served HTTP transport to land before anything else.
