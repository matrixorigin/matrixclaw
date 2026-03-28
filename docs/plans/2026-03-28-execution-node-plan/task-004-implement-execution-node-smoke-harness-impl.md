# Task 004: Implement execution node smoke harness

**depends-on**: task-004-write-execution-node-smoke-harness-test-test

## Goal
Ship the maintainer-facing smoke harness for the Execution Node milestone and ensure it is stable enough to become the model for later Node milestones.

## Files
- Modify or create: `scripts/verify-execution-node.sh`
- Modify related tests as needed
- Update `docs/long-horizon/Documentation.md` if commands or milestone notes change

## Verification
- `cargo fmt --all --check`
- `cargo test -p matrixclaw-app-host`
- `./scripts/verify-execution-node.sh`
