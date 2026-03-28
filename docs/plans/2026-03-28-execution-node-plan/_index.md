# Execution Node Implementation Plan

**Goal:** Implement the first concrete Node boundary in MatrixClaw by turning existing execution helpers into an `Execution Node` that the runtime can call without knowing local or sandbox backend details.

**Architecture:** Keep Gateway concerns untouched. Introduce a Node-oriented contract over the existing execution modules, route local/sandbox/disabled behavior through that contract, and then prove that runtime tool execution can reuse it. This should establish the pattern for later Screenshot, Browser, Filesystem, Camera, and Mouse Nodes.

**Design Support:**
- [Node Design Index](../2026-03-28-node-design/_index.md)
- [Node Architecture](../2026-03-28-node-design/architecture.md)
- [Node BDD Specs](../2026-03-28-node-design/bdd-specs.md)

## Execution Plan

```yaml
tasks:
  - id: "001-test"
    subject: "Write execution node contract test"
    slug: "write-execution-node-contract-test"
    type: "test"
    depends-on: []
  - id: "001-impl"
    subject: "Implement execution node contract"
    slug: "implement-execution-node-contract"
    type: "impl"
    depends-on: ["task-001-write-execution-node-contract-test-test"]
  - id: "002-test"
    subject: "Write execution node routing test"
    slug: "write-execution-node-routing-test"
    type: "test"
    depends-on: ["task-001-implement-execution-node-contract-impl"]
  - id: "002-impl"
    subject: "Implement execution node routing"
    slug: "implement-execution-node-routing"
    type: "impl"
    depends-on: ["task-002-write-execution-node-routing-test-test"]
  - id: "003-test"
    subject: "Write runtime execution node integration test"
    slug: "write-runtime-execution-node-integration-test"
    type: "test"
    depends-on: ["task-001-implement-execution-node-contract-impl", "task-002-implement-execution-node-routing-impl"]
  - id: "003-impl"
    subject: "Implement runtime execution node integration"
    slug: "implement-runtime-execution-node-integration"
    type: "impl"
    depends-on: ["task-003-write-runtime-execution-node-integration-test-test"]
  - id: "004-test"
    subject: "Write execution node smoke harness test"
    slug: "write-execution-node-smoke-harness-test"
    type: "test"
    depends-on: ["task-002-implement-execution-node-routing-impl", "task-003-implement-runtime-execution-node-integration-impl"]
  - id: "004-impl"
    subject: "Implement execution node smoke harness"
    slug: "implement-execution-node-smoke-harness"
    type: "impl"
    depends-on: ["task-004-write-execution-node-smoke-harness-test-test"]
```

## Task File References

- [Task 001 Test: Write execution node contract test](./task-001-write-execution-node-contract-test-test.md)
- [Task 001 Impl: Implement execution node contract](./task-001-implement-execution-node-contract-impl.md)
- [Task 002 Test: Write execution node routing test](./task-002-write-execution-node-routing-test-test.md)
- [Task 002 Impl: Implement execution node routing](./task-002-implement-execution-node-routing-impl.md)
- [Task 003 Test: Write runtime execution node integration test](./task-003-write-runtime-execution-node-integration-test-test.md)
- [Task 003 Impl: Implement runtime execution node integration](./task-003-implement-runtime-execution-node-integration-impl.md)
- [Task 004 Test: Write execution node smoke harness test](./task-004-write-execution-node-smoke-harness-test-test.md)
- [Task 004 Impl: Implement execution node smoke harness](./task-004-implement-execution-node-smoke-harness-impl.md)

## BDD Coverage

- `Runtime reaches execution through a Node boundary` -> task pair `001`
- `Execution Node routes local, sandboxed, and denied execution` -> task pair `002`
- `Tool-backed runtime execution reuses the Execution Node` -> task pair `003`
- `Execution Node establishes the pattern for future Nodes` -> task pair `004`

## Dependency Chain

```text
task-001-test -> task-001-impl
                    |
                    v
              task-002-test -> task-002-impl
                    |                 |
                    └--------┬--------┘
                             v
                    task-003-test -> task-003-impl
                             |
                             v
                    task-004-test -> task-004-impl
```

## Execution Handoff

This plan is the milestone execution artifact for Milestone 03 in `docs/long-horizon/Plans.md`.
Execute it before adding any new non-execution Node types.
