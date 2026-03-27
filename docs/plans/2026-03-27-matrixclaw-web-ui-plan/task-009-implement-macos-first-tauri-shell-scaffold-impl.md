# Task 009: Implement macOS-first Tauri shell scaffold

**depends-on**: task-009-write-tauri-shell-boundary-test-test

## Description

Implement the optional `Tauri 2` shell scaffold that wraps the loopback-served MatrixClaw UI for macOS first, without making the shell the core runtime.

## Execution Context

**Task Number**: 009 of 009  
**Phase**: Refinement  
**Prerequisites**: Red shell-boundary task exists

## BDD Scenario

```gherkin
Scenario: Desktop shell launches the same loopback UI boundary
  Given MatrixClaw has a browser-first local UI served by app-host
  When the macOS desktop shell launches
  Then it starts or attaches to the local app-host process
  And it renders the same web UI boundary used by the browser flow
  And Linux and Windows remain future shell targets through the same boundary
```

**Spec Source**: scope-specific scenario derived from the UI framework decision and `../2026-03-26-rust-openclaw-runtime-design/delivery-plan.md`

## Files to Modify/Create

- Modify: `apps/desktop-shell/package.json`
- Modify: `apps/desktop-shell/src/`
- Modify: `apps/desktop-shell/src-tauri/tauri.conf.json`
- Modify: `apps/desktop-shell/src-tauri/src/`
- Modify: `README.md`

## Steps

### Step 1: Re-run the failing shell-boundary task
- Confirm the shell contract remains unmet before implementation.

### Step 2: Implement the minimal Green scaffold
- Create a macOS-first `Tauri 2` shell that opens the loopback UI.
- Keep the shell thin; it should launch or attach, not absorb runtime logic.
- Document Linux/Windows as later shell targets rather than pretending they are first-class immediately.

### Step 3: Verify
- Run the shell-boundary verification.
- Re-run the broader build/test commands that remain relevant after introducing the shell scaffold.

## Verification Commands

```bash
test -f apps/desktop-shell/src-tauri/tauri.conf.json
cargo test -p matrixclaw-app-host
```

## Success Criteria

- macOS gets an optional native shell path.
- The underlying UI boundary remains loopback and browser-first.
- The shell scaffold does not re-centralize runtime ownership outside `app-host`.
