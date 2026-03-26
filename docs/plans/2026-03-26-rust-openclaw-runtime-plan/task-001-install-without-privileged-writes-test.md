# Task 001: [TEST] Install without privileged writes

**depends-on**: None

## Description

Create a failing operator-facing test that proves MatrixClaw installs into a user-owned path and does not assume `sudo`, Bun, Node.js, or Docker.

## Execution Context

**Task Number**: 001 of 019 (test)  
**Phase**: Setup  
**Prerequisites**: None. This task establishes the initial workspace, installer contract, and packaging test harness.

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

- Create: `Cargo.toml`
- Create: `crates/app-host/tests/install_without_privileged_writes.rs`
- Create: `crates/app-host/src/install.rs`
- Create: `scripts/install.sh`
- Create: `rust-toolchain.toml`

## Steps

### Step 1: Verify Scenario

- Confirm the install scenario text and outcomes match the design spec exactly.

### Step 2: Create the failing Red test

- Create a black-box installer test that runs `scripts/install.sh` against a temporary home directory.
- Stub external environment checks so the test proves “no dependency required” rather than touching real package managers.
- Assert on install destination, exit code, and `matrixclaw version` execution.
- Ensure the first failure is semantic, such as writing to `/usr/local/bin` or requiring absent tooling.

### Step 3: Lock the contract surface

- Add only the minimum installer entrypoint signatures and workspace manifests needed for the test to compile.
- Do not implement install behavior in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host install_without_privileged_writes -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- A single failing test exists for the install scenario.
- The failure demonstrates an unmet installer contract, not a missing-test-harness problem.
- No production install logic is implemented beyond explicit interfaces and skeleton files.
