# Documentation

Last updated: 2026-03-28

## What This Project Delivers
- MatrixClaw is a self-hosted agent host with a shared live runtime, embedded browser-first UI, OpenClaw-compatible served transports, and an emerging gateway model for external communication channels.
- The current architecture now explicitly distinguishes:
  - Gateways: communication boundaries for browser, OpenClaw, Matrix, and future channels
  - Nodes: future host capability boundaries for screenshots, browsing, camera, mouse, shell, filesystem, and similar powers

## Local Setup
- Prerequisites:
  - Rust toolchain
  - `pnpm` for the UI workspace
- Install:
  - `cargo build --release`
  - `MATRIXCLAW_SOURCE_BIN=target/release/matrixclaw ./scripts/install.sh`
- Start:
  - `~/.matrixclaw/bin/matrixclaw`
  - or `cargo run -p matrixclaw-app-host --bin matrixclaw -- serve --fixture demo`

## Verification Commands
- Lint:
  - `cargo fmt --all --check`
- Typecheck:
  - `pnpm --dir ui check`
- Tests:
  - `cargo test -p matrixclaw-app-host`
  - `cargo test -p matrixclaw-compat-openclaw`
- Build:
  - `pnpm --dir ui build`
- Scenario and smoke verification:
  - `pnpm --dir ui test:e2e`
  - `./scripts/verify-execution-node.sh`
  - `./scripts/verify-live-runtime.sh`
  - `./scripts/verify-served-transports.sh`
  - `./scripts/verify-matrix-gateway.sh`

## Architecture Snapshot
- Core modules:
  - shared runtime: [live_runtime.rs](/home/momo/src/matrixclaw/crates/app-host/src/live_runtime.rs)
  - normalized ingress: [ingress.rs](/home/momo/src/matrixclaw/crates/app-host/src/ingress.rs)
  - gateway layer: [gateway/](/home/momo/src/matrixclaw/crates/app-host/src/gateway)
  - OpenClaw served transport: [openclaw_transport.rs](/home/momo/src/matrixclaw/crates/app-host/src/openclaw_transport.rs)
  - browser and API host: [http/](/home/momo/src/matrixclaw/crates/app-host/src/http)
- Data flow:
  - external surface -> gateway or served transport -> ingress -> live runtime -> projected reply
- Operational constraints:
  - channel-specific retry/dedupe must stay outside the runtime
  - host/device abilities should converge into a Node layer rather than remain scattered helpers
  - browser, OpenClaw, and external gateways should continue sharing one session model

## Milestone Status
- Milestone 01:
  - shared runtime and served transport baseline complete
- Milestone 02:
  - Gateway model and Matrix gateway baseline complete
- Milestone 03:
  - Execution Node milestone in progress with a dedicated smoke harness: `./scripts/verify-execution-node.sh`
- Milestone 04:
  - real external connector lifecycle not yet implemented

## Runtime Execution Notes
- Last updated:
  - 2026-03-28
- Iteration:
  - planning baseline
- Reviewer outcome:
  - continue
- Reviewer assessment:
  - current architecture is coherent enough to proceed, but capability work now needs a formal Node boundary
- Optional verification status:
  - gateway/runtime verification green at the current checkpoint
- Validation summary:
  - `cargo fmt --all --check` passed
  - `cargo test -p matrixclaw-app-host` passed
  - `./scripts/verify-matrix-gateway.sh` passed
- Stop reason:
  - none

## Troubleshooting
- Issue: Gateway code starts absorbing host capability logic.
  Fix: move capability semantics into a Node-oriented module before continuing feature work.
- Issue: A new connector wants runtime-specific special cases.
  Fix: normalize it through ingress and keep protocol details in the gateway layer.
- Issue: Execution helpers remain scattered and hard to reason about.
  Fix: define the first concrete Node slice and refactor around that boundary instead of adding more one-off helpers.
