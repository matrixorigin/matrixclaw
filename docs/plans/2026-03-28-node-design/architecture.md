# Architecture

## Boundary

A `Node` is the host capability boundary below the runtime.

The runtime should ask Nodes to do work.
Nodes should execute against the host system, sandbox, or external capability surface.
Nodes should return structured results to the runtime.

Nodes are not Gateways.
They do not receive Matrix or OpenClaw messages, resolve workspaces, or send chat replies.

## Why a Node boundary is needed now

The current codebase already has the beginnings of a capability layer, but it is scattered:

- [execution.rs](/home/momo/src/matrixclaw/crates/app-host/src/execution.rs)
- [local_command.rs](/home/momo/src/matrixclaw/crates/app-host/src/local_command.rs)
- [sandbox_backend.rs](/home/momo/src/matrixclaw/crates/app-host/src/sandbox_backend.rs)
- [plugin_launcher.rs](/home/momo/src/matrixclaw/crates/app-host/src/plugin_launcher.rs)

These modules are useful, but they are still helper-oriented rather than boundary-oriented.

Without a Node model, future capabilities like screenshots, browser automation, camera access, mouse movement, and filesystem operations will likely land as more unrelated helpers. That would make policy, testing, and runtime orchestration harder over time.

## Layering

```text
Gateway
  -> Ingress
  -> Live Runtime
  -> Node
  -> Host system / sandbox / device capability
```

Responsibilities by layer:

1. `Gateway`
- external communication
- sender/channel/thread routing
- retry/dedupe/delivery
- workspace and session resolution

2. `Ingress`
- normalized internal message envelope

3. `Live runtime`
- session persistence
- queue semantics
- tool and turn orchestration
- streamed event emission

4. `Node`
- capability-specific request/response contracts
- policy and permission checks
- local vs sandbox routing
- capability execution details

## First concrete Node slice

The first Node slice should be `Execution Node`.

Reasons:
- the code already supports local and sandboxed execution
- the runtime already has tool execution concepts
- this is the shortest path to a real Node boundary with minimal churn
- it avoids prematurely mixing GUI/device concerns into the first capability model

`Execution Node` should absorb or orchestrate:
- local command execution
- sandbox backend routing
- execution policy and mode selection

Likely ownership:
- [execution.rs](/home/momo/src/matrixclaw/crates/app-host/src/execution.rs)
- [local_command.rs](/home/momo/src/matrixclaw/crates/app-host/src/local_command.rs)
- [sandbox_backend.rs](/home/momo/src/matrixclaw/crates/app-host/src/sandbox_backend.rs)

## Proposed Node model

Minimal shape first:

```text
NodeRequest
  capability
  workspace/cwd context
  policy context
  structured arguments

NodeResult
  capability
  backend used
  exit status or outcome status
  structured payload
  stderr/error details when relevant
```

Concrete first capability:

```text
ExecutionNodeRequest
  command
  args
  cwd
  preferred backend

ExecutionNodeResult
  backend
  exit_code
  stdout
  stderr
```

This maps cleanly onto what already exists today.

## Node policy split

The runtime should decide *that* it wants execution.
The Node layer should decide *how* that execution reaches the host under current policy.

Examples:
- local allowed -> run locally
- sandbox required -> route through sandbox backend
- execution disabled -> return policy denial

This keeps runtime orchestration separate from capability enforcement.

## Relationship to tools

Tools and Nodes should not become competing concepts.

Recommended model:
- tools are runtime-visible operations
- some tools are implemented by calling Nodes
- Nodes are the host boundary behind those tools

Example:
- `run_command` tool
  - implemented via `Execution Node`
- future `take_screenshot` tool
  - implemented via `Screenshot Node`
- future `browse_page` tool
  - implemented via `Browser Node`

## Relationship to plugins

Plugins may expose capabilities, but that does not remove the need for Nodes.

Two valid patterns:
- plugin exposes a tool contract directly
- plugin is launched behind a Node-style boundary

The important rule is the same:
- plugin process details should not leak into Gateway logic
- host capability details should not leak into Gateway logic

## Migration strategy

Phase 1:
- define `Execution Node` in docs
- keep existing modules mostly in place
- introduce a Node-oriented contract over the existing execution helpers

Phase 2:
- make the runtime call through the Node contract instead of ad hoc execution routing
- add tests around policy and backend selection through the Node

Phase 3:
- add sibling Nodes:
  - `Screenshot Node`
  - `Browser Node`
  - `Filesystem Node`
  - `Camera Node`
  - `Mouse Node`

## What not to do

- do not put Matrix/OpenClaw/browser delivery behavior into Node code
- do not make Nodes responsible for workspace routing from external messages
- do not invent a huge generic capability framework before one concrete Node works
- do not force all future capabilities into the first Node implementation

## Validation rule

The first Node milestone is complete only when:
- one real `Execution Node` contract exists
- the runtime reaches it through a stable boundary
- policy and backend routing happen in the Node layer
- focused tests prove local, sandboxed, and denied execution behavior through that Node
