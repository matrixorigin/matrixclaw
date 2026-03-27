# Task 009: Write Tauri shell boundary test

**depends-on**: task-003-implement-local-setup-server-and-shell-routing-impl

## Description

Add a failing test or contract fixture proving the optional desktop shell can launch or attach to the local `app-host` UI boundary without owning runtime logic itself.

## Execution Context

**Task Number**: 009 of 009  
**Phase**: Refinement  
**Prerequisites**: local setup/web shell routing exists

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

- Create: `apps/desktop-shell/src-tauri/`
- Create: `apps/desktop-shell/src/`
- Create: `apps/desktop-shell/package.json`
- Create: `apps/desktop-shell/src-tauri/tauri.conf.json`
- Create: `apps/desktop-shell/src-tauri/tests/` or contract fixture
- Modify: `README.md`

## Steps

### Step 1: Verify Scenario
- Confirm the desktop shell is optional and must wrap the existing loopback UI rather than replace it.

### Step 2: Create the failing Red test
- Add a shell-boundary test or contract fixture that asserts:
  - app-host startup or attach behavior
  - shell loads the loopback URL
  - shell does not duplicate runtime config logic

### Step 3: Lock the shell boundary
- Define the startup contract between the shell and `app-host`.
- Do not implement the actual shell behavior in this task.

## Verification Commands

```bash
test -f apps/desktop-shell/src-tauri/tauri.conf.json
```

## Success Criteria

- The optional shell boundary is specified before implementation.
- The shell is constrained to wrapping the existing UI boundary.
- The task creates a clear Red state for the later shell scaffold.
