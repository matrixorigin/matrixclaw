# Task 001: [IMPL] Install without privileged writes

**depends-on**: task-001-install-without-privileged-writes-test

## Description

Implement the initial Cargo workspace, versioned binary entrypoint, and shell installer so MatrixClaw installs into a user-owned path and runs `matrixclaw version` without external runtime prerequisites.

## Execution Context

**Task Number**: 001 of 019 (impl)  
**Phase**: Setup  
**Prerequisites**: The paired Red test exists and fails for the intended operator-facing reason.

## BDD Scenario

```gherkin
Scenario: User installs MatrixClaw without privileged writes
  Given a Linux or macOS machine without MatrixClaw installed
  And the user has a writable home directory
  When the user runs the install command
  Then the installer places the binary in a user-owned directory
  And the installer does not require Bun, Node.js, or Docker
  And the user can run `matrixclaw version` successfully
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Modify: `Cargo.toml`
- Create: `crates/app-host/Cargo.toml`
- Create: `crates/app-host/src/main.rs`
- Modify: `crates/app-host/src/install.rs`
- Modify: `scripts/install.sh`
- Create: `README.md`

## Steps

### Step 1: Re-run the failing test

- Confirm the installer test still fails before changing behavior.

### Step 2: Implement the minimal install path

- Define the app-host binary package and a `version` command.
- Implement installer path resolution to `~/.matrixclaw/bin` or equivalent user-owned location.
- Ensure the install script does not invoke Bun, Node.js, Docker, or privileged writes.
- Add explicit version output wiring needed by the test contract.

### Step 3: Verify Pass

- Run the targeted installer test and confirm it passes.

### Step 4: Regression sweep

- Run the app-host test package to ensure no setup regressions are introduced.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host install_without_privileged_writes -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- The targeted install scenario passes.
- MatrixClaw exposes a working `version` command.
- Installer behavior uses only user-owned paths and documented prerequisites.
