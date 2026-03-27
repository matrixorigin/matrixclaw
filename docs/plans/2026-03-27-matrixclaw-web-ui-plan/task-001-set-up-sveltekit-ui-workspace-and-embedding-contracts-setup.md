# Task 001: Set up SvelteKit UI workspace and embedding contracts

## Description

Create the frontend workspace, route/layout skeleton, Rust-side asset contract surfaces, and shared test/build entrypoints needed for an embedded web UI without requiring a Node process at runtime.

## Execution Context

**Task Number**: 001 of 008  
**Phase**: Setup  
**Prerequisites**: none

## BDD Scenario

```gherkin
Scenario: UI workspace builds static assets for embedding
  Given MatrixClaw will serve a browser-first UI from the Rust binary
  When the frontend workspace is initialized
  Then SvelteKit can build static assets for embedding
  And Rust-side code has a stable contract for locating those assets
  And no Node.js process is required at runtime
```

**Spec Source**: scope-specific scenario derived from `../2026-03-26-rust-openclaw-runtime-design/delivery-plan.md`

## Files to Modify/Create

- Create: `ui/package.json`
- Create: `ui/pnpm-lock.yaml` or equivalent lockfile chosen by the team
- Create: `ui/svelte.config.js`
- Create: `ui/vite.config.ts`
- Create: `ui/src/routes/+layout.svelte`
- Create: `ui/src/routes/+page.svelte`
- Create: `ui/src/routes/setup/+page.svelte`
- Create: `ui/src/routes/workspace/+page.svelte`
- Create: `ui/src/lib/`
- Create: `ui/static/`
- Create: `crates/app-host/src/ui_assets.rs`
- Modify: `crates/app-host/src/lib.rs`
- Modify: `crates/app-host/Cargo.toml`
- Modify: `README.md`
- Modify: `docs/plans/2026-03-27-matrixclaw-web-ui-plan/layout-diagrams.md`

## Steps

### Step 1: Verify Scenario
- Confirm the UI slice requires static asset embedding and browser-first setup from the design docs.

### Step 2: Create frontend workspace scaffolding
- Initialize a `SvelteKit` app configured for static output.
- Choose a single package manager and document it explicitly; prefer `pnpm`.
- Add route and layout skeletons that match the diagrams:
  - setup flow
  - workspace shell
  - left/center/right layout regions

### Step 3: Define Rust embedding contract
- Add a Rust-side module that defines where built UI assets are expected and how tests will locate fixture assets.
- Keep the contract explicit so later tasks can embed or fixture-swap assets without changing routing code.

### Step 4: Wire workspace-level developer commands
- Add documented build/test commands for the UI workspace.
- Ensure runtime execution does not depend on those dev commands.

## Verification Commands

```bash
test -f ui/package.json
test -f crates/app-host/src/ui_assets.rs
```

## Success Criteria

- The repo contains a dedicated `ui/` workspace.
- `app-host` has an explicit UI asset contract surface.
- Runtime assumptions are documented: build-time Node is acceptable, runtime Node is not.
