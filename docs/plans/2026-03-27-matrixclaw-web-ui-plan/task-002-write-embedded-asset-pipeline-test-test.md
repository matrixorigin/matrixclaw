# Task 002: Write embedded asset pipeline test

**depends-on**: task-001-set-up-sveltekit-ui-workspace-and-embedding-contracts-setup

## Description

Add a failing test proving `app-host` can load and serve built UI assets from a deterministic embedded or fixture-backed contract without starting a frontend dev server.

## Execution Context

**Task Number**: 002 of 008  
**Phase**: Foundation  
**Prerequisites**: frontend workspace and Rust asset contract exist

## BDD Scenario

```gherkin
Scenario: Embedded web UI shell is served from app-host
  Given MatrixClaw has built web UI assets
  When the local UI route is requested
  Then app-host serves the embedded shell document
  And browser refresh on a client-side route still resolves through the shell
  And no separate frontend runtime process is required
```

**Spec Source**: scope-specific scenario derived from `../2026-03-26-rust-openclaw-runtime-design/architecture.md`

## Files to Modify/Create

- Create: `crates/app-host/tests/embedded_ui_assets.rs`
- Modify: `crates/app-host/src/ui_assets.rs`
- Modify: `crates/app-host/src/lib.rs`

## Steps

### Step 1: Verify Scenario
- Re-state the asset serving expectations as test assertions before implementation.

### Step 2: Create the failing Red test
- Add a focused `app-host` test that uses fixture assets rather than a live frontend build.
- Assert shell asset resolution for `/` and at least one client-side route.
- Verify the test fails for a meaningful asset-routing reason.

### Step 3: Lock the serving contract
- Define any request/response or lookup interfaces needed by the test.
- Do not implement the actual serving behavior in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host embedded_ui_assets -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- One failing Red test captures embedded shell serving behavior.
- The failure is semantic, not caused by missing imports or compile breakage.
- Asset serving contracts are explicit enough for the Green task.
