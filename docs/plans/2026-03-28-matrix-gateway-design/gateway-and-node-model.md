# Gateway and Node Model

## Purpose

This document locks down the product vocabulary for external communication and host capabilities so future architecture work does not blur them together.

## Core definitions

### Gateway

A `Gateway` is how messages from outside systems enter and leave MatrixClaw.

Examples:
- Matrix gateway
- OpenClaw gateway
- browser gateway
- Telegram gateway
- Slack gateway

Responsibilities:
- receive inbound messages or events
- authenticate with the external system
- map sender/channel/thread identity into workspace and session context
- normalize inbound traffic into the internal ingress envelope
- project runtime replies back into channel-specific deliveries
- own retry, dedupe, delivery bookkeeping, and reply routing

Non-responsibilities:
- screenshots
- web browsing
- camera access
- mouse movement
- shell execution
- tool/capability semantics

### Node

A `Node` is a host-facing capability boundary that gives the runtime powers.

Examples:
- screenshot node
- browser node
- camera node
- mouse node
- shell node
- filesystem node

Responsibilities:
- expose a capability-specific request/response contract
- enforce permission and policy checks
- talk to the host OS, device, browser, or sandbox
- return structured results back to the runtime

Non-responsibilities:
- Matrix/Telegram/browser chat delivery
- workspace routing from external messages
- channel auth and retry semantics

### Runtime

The runtime is the planning and persistence layer between gateways and nodes.

Responsibilities:
- session ownership
- turn execution
- queue semantics
- persistence
- tool and node orchestration
- streamed event emission

The runtime should not know:
- Matrix room event ids
- Slack webhook retry rules
- browser tab control details
- camera device APIs

## Relationship

```text
external systems
  -> gateways
  -> ingress
  -> runtime
  -> nodes
  -> host system abilities
```

Replies flow back in reverse:

```text
host capability results
  -> nodes
  -> runtime
  -> gateways
  -> external systems
```

## Workspace model

Gateways are mainly related to workspaces because they decide where an external message belongs.

A gateway should be able to resolve:
- which workspace the message belongs to
- which session should resume
- which agent or target should receive the message
- where replies should be routed back

Nodes are not workspace routers.
They execute inside the context the runtime has already resolved.

## Design rules

1. Gateways own communication concerns.
2. Nodes own capability concerns.
3. Runtime stays in the middle and remains channel-agnostic and capability-agnostic.
4. Ingress remains an internal normalized contract, not a product-facing concept.
5. No gateway code should leak delivery or retry semantics into node execution.
6. No node code should leak host/device APIs into gateway routing.

## Current mapping in MatrixClaw

Today:
- `gateway/` is the beginning of the Gateway layer
- `ingress.rs` is the normalized internal handoff into the runtime
- `live_runtime.rs` is the shared runtime service

Later:
- screenshot, browser, camera, mouse, shell, and similar abilities should converge into a `node/` layer
- existing execution-related modules should be absorbed under that capability boundary instead of remaining as unrelated one-off modules

## Naming guidance

Good names:
- `Gateway`
- `GatewayRunner`
- `GatewayPort`
- `Node`
- `NodeExecutor`
- `CapabilityNode`

Avoid using channel-specific names for the generic abstraction itself.

Good:
- generic `GatewayPort`
- concrete `MatrixGateway`

Bad:
- generic abstraction named `MatrixClient`

Likewise for nodes:
- generic `NodeExecutor`
- concrete `ScreenshotNode`
- concrete `BrowserNode`
