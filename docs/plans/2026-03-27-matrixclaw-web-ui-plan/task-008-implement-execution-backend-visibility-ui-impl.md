# Task 008: Implement execution backend visibility UI

**depends-on**: task-008-write-execution-backend-visibility-test-test

## Description

Implement the setup, workspace, and detail-panel UI surfaces that expose execution provenance and sandbox backend priority, with `docker` first and `boxlite` second.

## Execution Context

**Task Number**: 008 of 009  
**Phase**: Core Features  
**Prerequisites**: Red execution-visibility test exists

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

- Modify: `crates/app-host/src/http/execution_api.rs`
- Modify: `ui/src/lib/execution/`
- Modify: `ui/src/routes/setup/+page.svelte`
- Modify: `ui/src/routes/workspace/+page.svelte`
- Optionally Modify: `crates/manifests/src/config.rs`

## Steps

### Step 1: Re-run the failing test
- Confirm the execution-visibility test still fails before implementation.

### Step 2: Implement the Green path
- Expose execution backend policy and actual backend-used state through `app-host`.
- Render backend badges and execution details in the workspace surface.
- Render sandbox choice and backend priority in setup/settings surfaces.
- Show hard “sandbox required but unavailable” failures explicitly.

### Step 3: Verify
- Run the focused execution-visibility test.
- Re-run `app-host` tests.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host execution_backend_visibility -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- Users can tell whether code ran via `local`, `docker`, or `boxlite`.
- Backend priority is visible and correct.
- Required-sandbox failures are explicit and non-silent.
