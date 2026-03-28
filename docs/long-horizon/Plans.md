# Plans

Last updated: 2026-03-28

## Verification Checklist
- [ ] `cargo fmt --all --check`
- [ ] `cargo test -p matrixclaw-app-host`
- [ ] `cargo test -p matrixclaw-compat-openclaw`
- [ ] `pnpm --dir ui check`
- [ ] `pnpm --dir ui build`
- [ ] `pnpm --dir ui test:e2e`
- [ ] `./scripts/verify-live-runtime.sh`
- [ ] `./scripts/verify-served-transports.sh`
- [ ] `./scripts/verify-matrix-gateway.sh`

## Milestones

### Milestone 01 - Shared Runtime and Served Transport Baseline
Scope:
- [x] One session-backed live runtime serves browser and OpenClaw paths.
- [x] Browser path supports real streaming over loopback.
- [x] Served OpenClaw HTTP and WebSocket paths reuse the same runtime/session model.

Key files/modules:
- [crates/app-host/src/live_runtime.rs](/home/momo/src/matrixclaw/crates/app-host/src/live_runtime.rs)
- [crates/app-host/src/ingress.rs](/home/momo/src/matrixclaw/crates/app-host/src/ingress.rs)
- [crates/app-host/src/openclaw_transport.rs](/home/momo/src/matrixclaw/crates/app-host/src/openclaw_transport.rs)
- [crates/app-host/src/http/agent_api.rs](/home/momo/src/matrixclaw/crates/app-host/src/http/agent_api.rs)
- [crates/app-host/src/http/openclaw_api.rs](/home/momo/src/matrixclaw/crates/app-host/src/http/openclaw_api.rs)

Acceptance criteria:
- [x] One runtime path persists and resumes sessions across browser and OpenClaw.
- [x] Streamed runtime events are exposed through both browser and OpenClaw transport surfaces.

Verification commands:
- `cargo test -p matrixclaw-app-host`
- `cargo test -p matrixclaw-compat-openclaw`
- `./scripts/verify-served-transports.sh`

Execution workflow:
- [ ] Objective-first loop runner
- [ ] Superpowers writing-plans
- [ ] Superpowers subagent-driven-development
- [x] Superpowers executing-plans
- [ ] Other: direct execution

Execution artifact:
- `docs/plans/2026-03-27-matrixclaw-live-agent-runtime-plan/`
- `docs/plans/2026-03-28-matrixclaw-served-transport-plan/`

Status:
- [ ] not started
- [ ] in progress
- [x] complete

### Milestone 02 - Gateway Model and Matrix Gateway Baseline
Scope:
- [x] Lock the Gateway model as the external communication boundary.
- [x] Ship Matrix-first inbound normalization, delivery projection, dedupe, retry, and shared-session smoke.
- [x] Add a fixture-backed gateway runner that exercises inbound receive and outbound streamed delivery.

Key files/modules:
- [crates/app-host/src/gateway/mod.rs](/home/momo/src/matrixclaw/crates/app-host/src/gateway/mod.rs)
- [crates/app-host/src/gateway/matrix.rs](/home/momo/src/matrixclaw/crates/app-host/src/gateway/matrix.rs)
- [crates/app-host/src/gateway/runtime.rs](/home/momo/src/matrixclaw/crates/app-host/src/gateway/runtime.rs)
- [crates/app-host/src/gateway/transport.rs](/home/momo/src/matrixclaw/crates/app-host/src/gateway/transport.rs)
- [docs/plans/2026-03-28-matrix-gateway-design/gateway-and-node-model.md](/home/momo/src/matrixclaw/docs/plans/2026-03-28-matrix-gateway-design/gateway-and-node-model.md)

Acceptance criteria:
- [x] Gateway concerns stay outside the live runtime.
- [x] Browser and Matrix share one persisted session model.
- [x] The product vocabulary distinguishes Gateways from Nodes.

Verification commands:
- `cargo fmt --all --check`
- `cargo test -p matrixclaw-app-host`
- `./scripts/verify-matrix-gateway.sh`

Execution workflow:
- [ ] Objective-first loop runner
- [ ] Superpowers writing-plans
- [ ] Superpowers subagent-driven-development
- [x] Superpowers executing-plans
- [ ] Other: direct execution

Execution artifact:
- `docs/plans/2026-03-28-matrix-gateway-plan/`
- `docs/plans/2026-03-28-matrix-gateway-design/`

Status:
- [ ] not started
- [ ] in progress
- [x] complete

### Milestone 03 - Node Boundary and First Host Capability Slice
Scope:
- [ ] Define the generic Node boundary for host/system abilities.
- [ ] Choose the first concrete Node slice and integrate it through the runtime without mixing it with gateway concerns.
- [ ] Reconcile existing execution-related modules into a Node-oriented model.

Key files/modules:
- [crates/app-host/src/execution.rs](/home/momo/src/matrixclaw/crates/app-host/src/execution.rs)
- [crates/app-host/src/local_command.rs](/home/momo/src/matrixclaw/crates/app-host/src/local_command.rs)
- [crates/app-host/src/sandbox_backend.rs](/home/momo/src/matrixclaw/crates/app-host/src/sandbox_backend.rs)
- [crates/app-host/src/plugin_launcher.rs](/home/momo/src/matrixclaw/crates/app-host/src/plugin_launcher.rs)
- future `crates/app-host/src/node/`

Acceptance criteria:
- [ ] A dedicated Node design doc exists and is linked from the long-horizon docs.
- [ ] The first Node-facing capability works end-to-end through the runtime.
- [ ] Gateway code remains free of host capability implementation details.

Verification commands:
- `cargo fmt --all --check`
- `cargo test -p matrixclaw-app-host`
- focused node smoke script or test to be added during milestone planning

Execution workflow:
- [ ] Objective-first loop runner
- [x] Superpowers writing-plans
- [ ] Superpowers subagent-driven-development
- [ ] Superpowers executing-plans
- [ ] Other: direct execution

Execution artifact:
- `pending`

Status:
- [ ] not started
- [x] in progress
- [ ] complete

### Milestone 04 - Real External Connector Lifecycle
Scope:
- [ ] Replace fixture-only gateway driving with a real external connector lifecycle for Matrix or another first-class channel.
- [ ] Add startup/config wiring for a real gateway process lifecycle.
- [ ] Keep all connector SDK or protocol specifics isolated behind the Gateway boundary.

Key files/modules:
- [crates/app-host/src/gateway/client.rs](/home/momo/src/matrixclaw/crates/app-host/src/gateway/client.rs)
- [crates/app-host/src/gateway/transport.rs](/home/momo/src/matrixclaw/crates/app-host/src/gateway/transport.rs)
- [crates/app-host/src/lib.rs](/home/momo/src/matrixclaw/crates/app-host/src/lib.rs)
- future connector-specific module(s)

Acceptance criteria:
- [ ] A real connector can receive an inbound message and send streamed replies through the gateway runner.
- [ ] Retry and dedupe semantics still live in the gateway layer only.
- [ ] The runtime and node layers remain connector-agnostic.

Verification commands:
- `cargo test -p matrixclaw-app-host`
- updated gateway smoke harness for the concrete connector

Execution workflow:
- [ ] Objective-first loop runner
- [x] Superpowers writing-plans
- [ ] Superpowers subagent-driven-development
- [ ] Superpowers executing-plans
- [ ] Other: direct execution

Execution artifact:
- `pending`

Status:
- [x] not started
- [ ] in progress
- [ ] complete

## Risk Register
- Risk: Gateway terminology is clear, but the Node boundary is not yet backed by code.
  Impact: high
  Mitigation: Make Node the next milestone before more capability work lands.
- Risk: Connector-specific code may creep into the gateway abstraction layer.
  Impact: medium
  Mitigation: Keep generic gateway ports small and push concrete SDK logic into connector modules only.
- Risk: Existing execution modules may not map cleanly into a Node model.
  Impact: medium
  Mitigation: Start with one concrete Node slice and refactor around working behavior rather than forcing a top-down rewrite.

## Decision Log
- 2026-03-28: MatrixClaw uses `Gateway` for external communication and `Node` for host abilities, reason: this cleanly separates messaging from powers and matches the desired product model.
- 2026-03-28: The current gateway port layer is transitional and should not define the final product vocabulary, reason: protocol-specific naming in the abstraction layer would age badly.
- 2026-03-28: Node boundary work is the next architectural milestone, reason: capability work should not continue as ad hoc execution helpers.

## Loop Status
- Run ID: not started
- Last updated: 2026-03-28
- Iteration: 0
- Status: planning baseline created
- Work status: milestones 01 and 02 complete, milestone 03 selected next
- Review decision: continue
- Validation: current gateway/runtime verification green at latest checkpoint
- Progress gate: passed
- Next step: write the Node boundary design and the first node-focused execution plan
- Stop reason: none

## Next Milestone
- Active milestone: Milestone 03 - Node Boundary and First Host Capability Slice
- Next action: create the Node design doc and milestone execution plan before implementing new host capabilities
