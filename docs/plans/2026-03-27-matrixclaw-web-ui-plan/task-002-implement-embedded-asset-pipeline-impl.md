# Task 002: Implement embedded asset pipeline

**depends-on**: task-002-write-embedded-asset-pipeline-test-test

## Description

Implement the minimal asset loading and shell fallback behavior required for `app-host` to serve built `SvelteKit` assets from an embedded or fixture-backed source.

## Execution Context

**Task Number**: 002 of 008  
**Phase**: Foundation  
**Prerequisites**: Red test exists for embedded UI asset serving

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

- Modify: `crates/app-host/src/ui_assets.rs`
- Modify: `crates/app-host/src/lib.rs`
- Modify: `crates/app-host/Cargo.toml`
- Optionally Create: `crates/app-host/src/http/assets.rs`

## Steps

### Step 1: Re-run the failing test
- Confirm the asset-serving test still fails before implementation.

### Step 2: Implement the minimal Green path
- Add deterministic loading for built or fixture assets.
- Serve the shell entry for `/` and client-side routes.
- Keep the behavior explicit so later HTTP/setup tasks can reuse it.

### Step 3: Run focused and broader verification
- Confirm the targeted test passes.
- Re-run `app-host` tests to protect existing install/setup behavior.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host embedded_ui_assets -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- `app-host` can resolve the shell entry without a frontend dev server.
- Client-side route fallback works.
- Existing `app-host` tests stay green.
