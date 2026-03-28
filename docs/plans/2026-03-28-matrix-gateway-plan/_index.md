# Matrix Gateway Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Load `superpowers:executing-plans` skill using the Skill tool to implement this plan task-by-task.

**Goal:** Implement a generic external gateway boundary over the shared ingress contract, then ship a Matrix-first gateway that can receive room messages, resume persisted sessions, and stream replies back without changing runtime semantics.

**Architecture:** Keep `matrixclaw-app-host` as the product host for runtime execution and served transports. Add a new `gateway` layer that owns Matrix-specific input parsing, session mapping, dedupe, retry, and outbound delivery, while delegating normalized requests through `ingress` into the existing `SessionBackedLiveRunService`. Prove the boundary with a Matrix-first gateway and a browser/Matrix shared-session smoke harness.

**Tech Stack:** Rust stable, Cargo workspace, `matrixclaw-app-host`, `matrixclaw-manifests`, `matrixclaw-session-runtime`, `tiny_http` loopback server, fixture-based gateway client doubles, `reqwest` for HTTP verification, shell smoke harnesses.

**Design Support:**
- [BDD Specs](../2026-03-28-matrix-gateway-design/bdd-specs.md)
- [Architecture](../2026-03-28-matrix-gateway-design/architecture.md)
- [Ingress Baseline](../2026-03-28-matrixclaw-served-transport-plan/_index.md)

## Context

MatrixClaw now has the correct internal transport shape: one live runtime, one session store, one ingress normalization contract, and proven reuse across browser and OpenClaw served transports. What it still does not have is a real IM-style gateway that validates this architecture against inbound events, outbound delivery routing, retry behavior, and session mapping.

This phase should prove that the current architecture scales beyond served transports. The gateway layer must stay fully outside the runtime core, and Matrix should be the first real connector because room/thread semantics map naturally onto the ingress contract and future gateway needs.

| Aspect | Current State | Target State |
|--------|--------------|--------------|
| External channel support | Browser and OpenClaw only | Generic gateway boundary plus Matrix-first connector |
| Inbound normalization | Served transport ingress only | Gateway events normalized into the same ingress envelope |
| Session reuse | Browser/OpenClaw proven | Browser/Matrix reuse proven through one persisted session model |
| Delivery projection | SSE and OpenClaw frame projection only | Matrix room/thread delivery projection over streamed runtime events |
| Retry and dedupe | Not modeled for real IM gateways | Gateway-owned retry/dedupe state outside the runtime core |
| Startup/config | No gateway settings | Optional Matrix gateway settings and disabled-by-default startup |
| Validation | Served-transport harness only | Matrix gateway smoke harness plus full app-host verification |

## Execution Plan

```yaml
tasks:
  - id: "001-test"
    subject: "Write gateway adapter contract test"
    slug: "write-gateway-adapter-contract-test"
    type: "test"
    depends-on: []
  - id: "001-impl"
    subject: "Implement gateway adapter contract"
    slug: "implement-gateway-adapter-contract"
    type: "impl"
    depends-on: ["task-001-write-gateway-adapter-contract-test-test"]
  - id: "002-test"
    subject: "Write Matrix ingress normalization test"
    slug: "write-matrix-ingress-normalization-test"
    type: "test"
    depends-on: ["task-001-implement-gateway-adapter-contract-impl"]
  - id: "002-impl"
    subject: "Implement Matrix ingress normalization"
    slug: "implement-matrix-ingress-normalization"
    type: "impl"
    depends-on: ["task-002-write-matrix-ingress-normalization-test-test"]
  - id: "003-test"
    subject: "Write Matrix streamed delivery test"
    slug: "write-matrix-streamed-delivery-test"
    type: "test"
    depends-on: ["task-001-implement-gateway-adapter-contract-impl", "task-002-implement-matrix-ingress-normalization-impl"]
  - id: "003-impl"
    subject: "Implement Matrix streamed delivery"
    slug: "implement-matrix-streamed-delivery"
    type: "impl"
    depends-on: ["task-003-write-matrix-streamed-delivery-test-test"]
  - id: "004-test"
    subject: "Write gateway dedupe and retry boundary test"
    slug: "write-gateway-dedupe-and-retry-boundary-test"
    type: "test"
    depends-on: ["task-001-implement-gateway-adapter-contract-impl", "task-002-implement-matrix-ingress-normalization-impl"]
  - id: "004-impl"
    subject: "Implement gateway dedupe and retry boundary"
    slug: "implement-gateway-dedupe-and-retry-boundary"
    type: "impl"
    depends-on: ["task-004-write-gateway-dedupe-and-retry-boundary-test-test"]
  - id: "005-test"
    subject: "Write optional Matrix gateway startup test"
    slug: "write-optional-matrix-gateway-startup-test"
    type: "test"
    depends-on: []
  - id: "005-impl"
    subject: "Implement optional Matrix gateway startup"
    slug: "implement-optional-matrix-gateway-startup"
    type: "impl"
    depends-on: ["task-005-write-optional-matrix-gateway-startup-test-test", "task-002-implement-matrix-ingress-normalization-impl", "task-004-implement-gateway-dedupe-and-retry-boundary-impl"]
  - id: "006-test"
    subject: "Write browser Matrix session reuse smoke test"
    slug: "write-browser-matrix-session-reuse-smoke-test"
    type: "test"
    depends-on: ["task-003-implement-matrix-streamed-delivery-impl", "task-004-implement-gateway-dedupe-and-retry-boundary-impl", "task-005-implement-optional-matrix-gateway-startup-impl"]
  - id: "006-impl"
    subject: "Implement browser Matrix session reuse smoke harness"
    slug: "implement-browser-matrix-session-reuse-smoke-harness"
    type: "impl"
    depends-on: ["task-006-write-browser-matrix-session-reuse-smoke-test-test"]
```

## Task File References

- [Task 001 Test: Write gateway adapter contract test](./task-001-write-gateway-adapter-contract-test-test.md)
- [Task 001 Impl: Implement gateway adapter contract](./task-001-implement-gateway-adapter-contract-impl.md)
- [Task 002 Test: Write Matrix ingress normalization test](./task-002-write-matrix-ingress-normalization-test-test.md)
- [Task 002 Impl: Implement Matrix ingress normalization](./task-002-implement-matrix-ingress-normalization-impl.md)
- [Task 003 Test: Write Matrix streamed delivery test](./task-003-write-matrix-streamed-delivery-test-test.md)
- [Task 003 Impl: Implement Matrix streamed delivery](./task-003-implement-matrix-streamed-delivery-impl.md)
- [Task 004 Test: Write gateway dedupe and retry boundary test](./task-004-write-gateway-dedupe-and-retry-boundary-test-test.md)
- [Task 004 Impl: Implement gateway dedupe and retry boundary](./task-004-implement-gateway-dedupe-and-retry-boundary-impl.md)
- [Task 005 Test: Write optional Matrix gateway startup test](./task-005-write-optional-matrix-gateway-startup-test-test.md)
- [Task 005 Impl: Implement optional Matrix gateway startup](./task-005-implement-optional-matrix-gateway-startup-impl.md)
- [Task 006 Test: Write browser Matrix session reuse smoke test](./task-006-write-browser-matrix-session-reuse-smoke-test-test.md)
- [Task 006 Impl: Implement browser Matrix session reuse smoke harness](./task-006-implement-browser-matrix-session-reuse-smoke-harness-impl.md)

## BDD Coverage

- `External channel event is normalized before entering the runtime` → task pair `001`
- `Matrix room message resumes the mapped persisted session` → task pair `002`
- `Matrix gateway streams assistant progress without changing runtime semantics` → task pair `003`
- `Delivery retries and dedupe stay outside the runtime` → task pair `004`
- `Matrix gateway remains disabled without explicit configuration` → task pair `005`
- `Browser and Matrix gateway share one persisted session model` → task pair `006`

All gateway work in this plan preserves one runtime rule:
- connector-specific input, retry, dedupe, and delivery stay in the gateway layer
- normalized ingress is the only boundary into the live runtime

## Dependency Chain

```text
task-001-test → task-001-impl
      │               │
      │               ├─→ task-002-test → task-002-impl
      │               │                         │
      │               │                         ├─→ task-003-test → task-003-impl
      │               │                         ├─→ task-004-test → task-004-impl
task-005-test ────────┘                         │                 │
                                                └────────────┬────┘
                                                             ↓
                                                   task-005-impl
                                                             │
task-003-impl ────────────────────────────────────────────────┤
task-004-impl ────────────────────────────────────────────────┤
task-005-impl ────────────────────────────────────────────────┴─→ task-006-test → task-006-impl
```

**Analysis**:
- task pair `001` establishes the generic gateway contract before any Matrix-specific work lands
- task pairs `002` and `004` can proceed in parallel once the shared gateway boundary exists
- task pair `003` waits for Matrix normalization because delivery routing depends on room/thread/session mapping
- task pair `005` stays independent on the Red side, but its Green side depends on real Matrix gateway pieces existing
- task pair `006` is the final product proof and should not start until delivery, retry, and startup wiring are all green

## Execution Handoff

**Plan draft complete and saved to `docs/plans/2026-03-28-matrix-gateway-plan/`. Review options:**

**1. Plan Review First (Recommended)** - review the task graph and gateway boundaries before commit.

**2. Orchestrated Execution After Approval** - use `superpowers:executing-plans`.

**3. Focused First Slice** - execute task pair `001` first if you want to lock the generic gateway contract before the Matrix adapter work begins.
