# Documentation

Last updated: 2026-03-28

## What This Project Delivers
- MatrixClaw is a self-hosted agent host with a shared live runtime, a Tauri-first macOS product shell, an embedded Svelte UI, OpenClaw-compatible served transports, and an emerging gateway model for external communication channels.
- The current architecture now explicitly distinguishes:
  - Gateways: communication boundaries for browser, OpenClaw, Matrix, and future channels
  - Nodes: future host capability boundaries for screenshots, browsing, camera, mouse, shell, filesystem, and similar powers

## Local Setup
- Prerequisites:
  - Rust toolchain
  - `pnpm` for the UI workspace
- Install:
  - `cargo build --release`
  - `MATRIXCLAW_SOURCE_BIN=target/release/zstar ./scripts/install.sh`
- Start:
  - `~/.zstar/bin/zstar`
  - or `cargo run -p zstar-app-host --bin zstar -- serve --fixture demo`

Important current limitation:
- the standalone binary install path is not yet a final-product experience because UI assets still depend on the current source-tree build output
- the active milestone is fixing this by making the Tauri app the primary packaged product

## Verification Commands
- Lint:
  - `cargo fmt --all --check`
- Typecheck:
  - `pnpm --dir ui check`
- Tests:
  - `cargo test -p zstar-app-host`
  - `cargo test -p zstar-compat-openclaw`
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
  - shared runtime: [live_runtime.rs](/home/momo/src/zstar/crates/app-host/src/live_runtime.rs)
  - normalized ingress: [ingress.rs](/home/momo/src/zstar/crates/app-host/src/ingress.rs)
  - gateway layer: [gateway/](/home/momo/src/zstar/crates/app-host/src/gateway)
  - OpenClaw served transport: [openclaw_transport.rs](/home/momo/src/zstar/crates/app-host/src/openclaw_transport.rs)
  - browser and API host: [http/](/home/momo/src/zstar/crates/app-host/src/http)
  - desktop shell boundary: [apps/desktop-shell/](/home/momo/src/zstar/apps/desktop-shell)
- Data flow:
  - external surface -> gateway or served transport -> ingress -> live runtime -> projected reply
- Operational constraints:
  - channel-specific retry/dedupe must stay outside the runtime
  - host/device abilities should converge into a Node layer rather than remain scattered helpers
  - browser, OpenClaw, and external gateways should continue sharing one session model
  - packaged product launch must not depend on repo-relative UI build paths

## Milestone Status
- Milestone 01:
  - shared runtime and served transport baseline complete
- Milestone 02:
  - Gateway model and Matrix gateway baseline complete
- Milestone 03:
  - Tauri product boundary and desktop app shell in progress
- Milestone 04:
  - Node boundary and first host capability slice not yet started
- Milestone 05:
  - real external connector lifecycle not yet implemented

## Runtime Execution Notes
- Last updated:
  - 2026-03-28
- Iteration:
  - planning baseline
- Reviewer outcome:
  - continue
- Reviewer assessment:
  - current runtime and gateway architecture is coherent enough to proceed, but the next risk is product incoherence: packaging, shell layout, and onboarding need to be rebuilt around a real Tauri app boundary
- Optional verification status:
  - gateway/runtime verification green at the current checkpoint
- Validation summary:
  - `cargo fmt --all --check` passed
  - `cargo test -p zstar-app-host` passed
  - `./scripts/verify-matrix-gateway.sh` passed
- Stop reason:
  - none

## Troubleshooting
- Issue: Installed binary fails with `setup shell asset not found`.
  Fix: stop depending on repo-relative `ui/build` assets and ship bundled UI assets through the Tauri product boundary.
- Issue: The app feels like a preview site instead of a desktop tool.
  Fix: replace the current landing-header shell with a persistent single-window app shell that owns navigation, workspace, inspector, and status surfaces.
- Issue: Gateway code starts absorbing host capability logic.
  Fix: move capability semantics into a Node-oriented module before continuing feature work.
- Issue: A new connector wants runtime-specific special cases.
  Fix: normalize it through ingress and keep protocol details in the gateway layer.
- Issue: Execution helpers remain scattered and hard to reason about.
  Fix: define the first concrete Node slice and refactor around that boundary instead of adding more one-off helpers.
