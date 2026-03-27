# Task 008: Write execution backend visibility test

**depends-on**: task-003-implement-local-setup-server-and-shell-routing-impl

## Description

Add a failing test proving the UI contract exposes execution provenance and sandbox backend policy, specifically distinguishing `local`, `docker`, and `boxlite`.

## Execution Context

**Task Number**: 008 of 009  
**Phase**: Core Features  
**Prerequisites**: local shell routing and API base exist

## BDD Scenario

```gherkin
Scenario: Execution surface reflects local vs docker vs boxlite backend
  Given MatrixClaw has execution policy and tool execution results
  When the user views a run that used local or sandboxed execution
  Then the UI shows whether execution was local, docker, or boxlite
  And sandbox policy shows docker as priority 1 and boxlite as priority 2
  And sandbox-required failures are shown explicitly instead of silently falling back
```

**Spec Source**: `./layout-diagrams.md`

## Files to Modify/Create

- Create: `crates/app-host/tests/execution_backend_visibility.rs`
- Create: `crates/app-host/src/http/execution_api.rs`
- Create: `ui/src/lib/execution/`
- Modify: `ui/src/routes/workspace/+page.svelte`
- Modify: `ui/src/routes/setup/+page.svelte`
- Modify: `docs/plans/2026-03-27-matrixclaw-web-ui-plan/layout-diagrams.md`

## Steps

### Step 1: Verify Scenario
- Confirm the product now requires execution provenance as a visible UI concern.

### Step 2: Create the failing Red test
- Assert the API exposes execution backend labels and sandbox policy ordering.
- Assert failure responses distinguish “sandbox required but unavailable” from ordinary execution failure.
- Assert the returned state supports rendering `local`, `docker`, and `boxlite` distinctly.

### Step 3: Lock execution visibility contracts
- Define request/response or event shapes for:
  - backend used
  - backend priority
  - fallback policy
  - hard sandbox-unavailable failure
- Do not implement real UI rendering in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host execution_backend_visibility -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- A Red test captures execution provenance and backend-priority visibility.
- `docker` and `boxlite` are explicit product-facing values.
- Silent fallback is prevented by the contract.
