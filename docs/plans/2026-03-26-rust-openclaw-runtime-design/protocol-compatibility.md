# Protocol Compatibility

## Purpose

This document defines what MatrixClaw should mean by OpenClaw client and protocol compatibility.

It does not cover skill or plugin import. That is handled by [Ecosystem Compatibility](./ecosystem-compatibility.md).

The goal is practical interoperability:

- existing OpenClaw-oriented clients can connect
- session and chat flows behave predictably
- compatibility claims are capability-based and testable

Not the goal:

- perfect reimplementation of every historical OpenClaw server quirk

## Compatibility Principles

### 1. Compatibility is a boundary contract

The compatibility layer translates between:

- external WebSocket or HTTP frames
- internal MatrixClaw session and run operations

Internal crates should not import OpenClaw protocol types directly.

### 2. Capability claims must be explicit

MatrixClaw should never say only “OpenClaw-compatible”.

It should publish a matrix such as:

- `connect`
- `agents.list`
- `chat.start`
- `chat.stream`
- `sessions.get`
- `sessions.export`

Each capability gets:

- support status
- supported version window
- known deviations

### 3. Fixtures over guesswork

Every supported capability needs:

- captured request fixtures
- expected response fixtures
- stream-frame fixtures where applicable

### 4. Backward pressure stays at the edge

If a client expects an odd frame shape, the adapter absorbs that complexity.
The runtime model does not change just to look like that client internally.

## Compatibility Surface

Initial compatibility surfaces:

- WebSocket control and streaming API
- HTTP endpoints needed by common clients
- auth and token handling
- agent discovery
- chat initiation and event streaming
- session export and basic import

## Capability Matrix

Suggested initial matrix:

| Capability | Status target | Notes |
|---|---|---|
| `connect` | `required` | handshake, auth, capabilities |
| `agents.list` | `required` | return available agent definitions |
| `chat.start` | `required` | create or target a session and begin a run |
| `chat.stream` | `required` | translate internal events into client-visible stream frames |
| `chat.cancel` | `required` | best-effort cancellation |
| `sessions.get` | `important` | basic session metadata and transcript retrieval |
| `sessions.export` | `important` | export durable transcript and metadata |
| `sessions.import` | `optional` | may start as tool-assisted migration rather than full wire parity |
| `plugins.list` | `optional` | compatibility only if a real client depends on it |

## Session Mapping

Compatibility clients may think in OpenClaw sessions, agents, or channels.
MatrixClaw must map those concepts onto its own runtime primitives.

Recommended mapping:

- external agent id -> internal agent profile or workspace configuration
- external session id -> MatrixClaw session id
- external chat request -> `session-runtime` inbound message plus run request
- external stream -> internal event stream projection

## Auth Model

Initial compatibility auth should be simple and local-first.

Recommended baseline:

- bearer token or compatibility API token
- explicit config switch to enable compatibility server
- loopback-only bind by default

Future options:

- multi-user auth
- per-agent tokens
- reverse-proxy integration

## WebSocket Design

The WebSocket adapter should be event driven rather than request-thread driven.

## Connection flow

Suggested steps:

1. client connects
2. server issues or accepts auth handshake
3. server returns capability descriptor
4. client requests agent list or starts chat
5. server subscribes client to translated run events

## Stream translation rules

Internal events must map consistently to external frames.

Examples:

- `message_delta` -> streamed content frame
- `message_completed` -> assistant message completion frame
- `tool_execution_started` -> tool-call frame
- `tool_execution_progress` -> progress/update frame where protocol allows
- `tool_execution_completed` -> tool-result frame
- `run_completed` -> terminal completion frame
- `run_failed` -> terminal error frame

## Critical rule

External stream frames must come from the same underlying generation that produced the persisted assistant message.

No separate non-streaming probe call is allowed just to decide whether streaming should happen.

## HTTP Design

HTTP compatibility should stay smaller than the internal API surface.

Recommended initial endpoints:

- `GET /compat/openclaw/agents`
- `POST /compat/openclaw/chat`
- `POST /compat/openclaw/chat/:sessionId/cancel`
- `GET /compat/openclaw/sessions/:sessionId`
- `GET /compat/openclaw/sessions/:sessionId/export`

Implementation rule:

- these endpoints are adapters over `session-runtime`
- they are not a second orchestration layer

## Capability Versioning

MatrixClaw should version compatibility by capability set, not by vague branding.

Suggested response shape:

```json
{
  "compat": {
    "protocol": "openclaw",
    "version": "0.1",
    "capabilities": {
      "connect": "supported",
      "agents.list": "supported",
      "chat.start": "supported",
      "chat.stream": "supported",
      "chat.cancel": "supported",
      "sessions.export": "supported",
      "sessions.import": "partial"
    }
  }
}
```

## Error Mapping

External clients need stable compatibility errors even when internal causes vary.

Suggested external error classes:

- `unauthorized`
- `unsupported_capability`
- `invalid_request`
- `rate_limited`
- `session_not_found`
- `run_conflict`
- `internal_error`

Suggested mapping rule:

- preserve high-level cause externally
- keep detailed diagnostics in logs
- avoid leaking internal stack traces or local paths by default

## Transcript And Export Semantics

Compatibility exports should come from the durable transcript, not from ephemeral UI cache.

Export should include:

- messages
- tool results
- session metadata
- compaction markers where relevant
- source compatibility metadata when the session originated from imported data

## Known Non-Goals For v1

- emulating every legacy OpenClaw bug
- supporting undocumented or deprecated frames without real client fixtures
- forcing MatrixClaw message roles to mirror OpenClaw exactly in storage

## Test Strategy

Compatibility is only credible with dedicated tests.

## Fixture sources

- real client capture from OpenClaw-compatible clients you care about
- server capture from reference implementations where behavior is stable
- MatrixClaw-generated expected fixtures for supported capabilities

## Required test classes

### Handshake tests

- auth accepted
- auth rejected
- capability listing

### Request translation tests

- `agents.list`
- `chat.start`
- `chat.cancel`

### Stream parity tests

Assert that:

- the external stream matches internal `message_delta` and `message_completed` events
- the final assistant message in exported transcript matches the stream
- tool frames line up with tool execution events

### Deviation tests

If MatrixClaw knowingly differs from OpenClaw, encode that difference in explicit tests and documentation.

## Rollout Strategy

Initial rollout should be narrow and honest.

Phase 1:

- loopback-only compatibility server
- `connect`
- `agents.list`
- `chat.start`
- `chat.stream`
- `chat.cancel`

Phase 2:

- session retrieval and export
- capability negotiation improvements

Phase 3:

- optional import helpers
- broader client matrix

## Design Checks

The protocol design is healthy if:

- compatibility claims can be listed capability by capability
- the runtime core stays unaware of protocol frames
- the same generation powers stream output and durable transcript
- unsupported client behavior fails explicitly instead of degrading silently
