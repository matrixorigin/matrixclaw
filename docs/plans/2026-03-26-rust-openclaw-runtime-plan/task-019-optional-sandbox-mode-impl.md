# Task 019: [IMPL] Optional sandbox mode

**depends-on**: task-019-optional-sandbox-mode-test

## Description

Implement opt-in sandbox execution routing so isolated commands use the configured backend and return structured results to the runtime.

## Execution Context

**Task Number**: 019 of 019 (impl)  
**Phase**: Operations  
**Prerequisites**: The paired Red test fails because sandbox routing is absent or bypassed.

## BDD Scenario

```gherkin
Scenario: Optional sandbox mode is enabled explicitly
  Given the user enables sandboxed execution in configuration
  When a tool requires isolated command execution
  Then MatrixClaw routes that command through the configured sandbox backend
  And returns structured execution results to the agent loop
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Modify: `crates/app-host/src/execution.rs`
- Modify: `crates/app-host/src/sandbox_backend.rs`
- Modify: `crates/manifests/src/config.rs`
- Modify: `crates/agent-core/src/tool.rs`

## Steps

### Step 1: Re-run the failing test

- Confirm the optional-sandbox test still fails before implementation.

### Step 2: Implement minimal sandbox routing

- Select the sandbox backend only when configuration enables it.
- Route isolated command execution through the sandbox adapter.
- Normalize backend responses into the structured result shape consumed by the runtime.

### Step 3: Verify Pass

- Run the targeted sandbox-mode test and confirm it passes.

### Step 4: Regression sweep

- Re-run app-host tests to protect both local and sandboxed execution paths.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host optional_sandbox_mode -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- Sandboxed execution occurs only when explicitly configured.
- Structured execution results flow back to the runtime consistently.
- The targeted scenario passes.
