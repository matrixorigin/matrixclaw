# Prompt

Last updated: 2026-03-28

## Objective
- Build MatrixClaw into a durable self-hosted agent host with one shared live runtime, clear Gateway and Node boundaries, browser and OpenClaw transport reuse, and real external gateway connectors without letting channel-specific code leak into the runtime core.

## Why This Matters
- Users want MatrixClaw to behave like an installable OpenClaw-class system, not a demo shell.
- The architecture must support many communication channels and many host capabilities without reworking the runtime for each new integration.
- Locking the Gateway and Node model now prevents the codebase from collapsing into transport-specific hacks and ad hoc capability modules later.

## Acceptance Criteria
- [ ] Browser, OpenClaw, and future external channels enter the system through Gateway boundaries that normalize into one ingress/runtime path.
- [ ] Host abilities such as screenshots, browsing, camera, mouse, shell, and filesystem are modeled as Nodes rather than hidden transport concerns.
- [ ] Long-horizon control docs stay current enough that execution can resume cleanly across milestones.
- [ ] Milestone plans, verification evidence, and shipped behavior are reconciled back into the control docs as work lands.

## Scope
In scope:
- [ ] Maintain the shared runtime and served transport architecture.
- [ ] Evolve external communication around Gateway terminology and boundaries.
- [ ] Evolve host capabilities around a future Node boundary.
- [ ] Implement real connector and capability milestones incrementally without breaking the runtime core.

Out of scope:
- [ ] Full production hardening for every connector and capability on the first pass.
- [ ] Multi-tenant authorization redesign.
- [ ] Native macOS/Windows product packaging beyond already-planned slices.

## Constraints
- Environment: Linux development environment, with macOS prioritized over Linux over Windows for eventual product polish.
- Tooling: Rust stable, Cargo workspace, SvelteKit UI, local loopback runtime, fixture-based integration tests, Playwright for browser smoke.
- Time: Continue iteratively without restarting architecture from scratch.
- Compliance/security: Keep gateway retry/dedupe and channel details outside the runtime; keep future host/device powers behind explicit Node boundaries and policy checks.

## Assumptions and Unknowns
Assumptions:
- [ ] Browser and OpenClaw served transports remain the baseline proof that one runtime can serve multiple communication surfaces.
- [ ] Matrix is still the first external IM-style gateway proving case.
- [ ] Node is the right product term for host capabilities, despite possible ambiguity with Node.js, because the architectural split is more important than the naming collision.

Unknowns:
- [ ] What the first concrete Node milestone should be: screenshot/browser first or shell/filesystem first.
- [ ] How broad the first real Matrix connector should be before SDK-specific hardening becomes worth it.
- [ ] Whether a generic node contract should be introduced before the first concrete Node implementation, or derived from the first capability slice.
