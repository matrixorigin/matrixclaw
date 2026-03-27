# Security And Operations

## Purpose

This document defines the operational trust model for MatrixClaw.

The product goal is easy local installation, but that only works if operators also understand:

- what code is trusted
- what code is isolated
- what gets downloaded
- what gets logged
- how upgrades and failures behave

## Trust Boundaries

MatrixClaw has several different trust zones.

They should not be collapsed into one “plugin” or “tool” bucket.

## Zone 1: Core binary

Includes:

- Rust runtime
- embedded web assets
- built-in tools
- built-in compatibility adapters

Trust level:

- highest

Operational expectation:

- signed releases
- reproducible or at least auditable build pipeline
- explicit version reporting

## Zone 2: Managed assets

Includes:

- browser engines
- OCR or STT models
- optional bridge runtimes

Trust level:

- high, but separately versioned from the core binary

Requirements:

- checksum verification
- source URL provenance
- removable independently of binary upgrades

## Zone 3: Imported skills

Includes:

- `SKILL.md`
- prompt bundles
- workspace context files

Trust level:

- medium

Notes:

- skills may influence model behavior
- they are usually not executable code by themselves
- provenance still matters because prompt injection can be packaged as a skill

## Zone 4: Plugins and MCP servers

Includes:

- subprocess plugins
- JSON-RPC tools
- MCP servers
- optional bridge-hosted plugins

Trust level:

- lower than built-in code

Requirements:

- explicit install provenance
- permissions and runtime hints in manifest
- visible operator controls

## Zone 5: Compatibility clients

Includes:

- WebSocket clients
- local web UI
- future remote API clients

Trust level:

- variable

Requirements:

- explicit auth
- loopback-only by default
- request logging and rate limiting

## Security Principles

### 1. Local-first does not mean trust everything

Even on a local machine, imported plugins and bridge runtimes are a separate trust domain from the core binary.

### 2. Make risk visible

When a user installs an artifact, MatrixClaw should show:

- origin
- support tier
- runtime type
- requested permissions

### 3. Secure defaults, optional escalation

Default behaviors should favor:

- user-owned install paths
- loopback binds
- no remote exposure
- no bridge runtime enabled unless requested

### 4. Explicit provenance everywhere

Every imported or downloaded artifact should record:

- source
- version or revision
- checksum when available
- install time
- importer version

## Permission Model

Initial permission categories:

- filesystem
- network
- process_spawn
- environment
- browser_automation
- workspace_write

These can begin as descriptive metadata even before full enforcement exists.

The important rule is to make them part of manifests and logs from day one.

## Tool And Plugin Isolation

## Built-in tools

Built-in tools run with the privileges of the MatrixClaw process unless a sandbox backend is enabled.

Requirements:

- clear docs
- timeout and cancellation support
- structured stdout/stderr/result capture

## External plugins

External plugins should run out of process by default.

Recommended controls:

- explicit command path
- explicit environment allowlist
- optional working directory restrictions
- timeout and restart policy

## Bridge runtime

If bridge support exists for Node or Bun artifacts:

- bridge runtime must be off by default
- bridge plugins must be labeled `bridge_only`
- failures in the bridge must not compromise core runtime startup

## Sandboxing Strategy

MatrixClaw should support useful operation without Docker.

Suggested execution modes:

- `local`
  - no extra sandbox, best local ergonomics
- `sandboxed`
  - optional backend such as container, jail, namespace, or VM
- `disabled`
  - execution tools unavailable by policy

Important rule:

- sandbox mode is an operator policy, not an install requirement

## Network Exposure

Default network posture:

- bind web UI and compatibility APIs to `127.0.0.1`
- require explicit config to expose on LAN or WAN
- generate API tokens rather than assuming trust from origin alone

If remote exposure is enabled, recommend:

- reverse proxy
- TLS termination
- IP allowlists where practical

## Secrets Handling

MatrixClaw will handle provider credentials and possibly plugin secrets.

Requirements:

- store secrets separately from public manifests where possible
- redact secrets from logs
- never include raw secrets in exported session artifacts
- support environment-variable references and future secret store integration

## Managed Asset Integrity

Managed downloads are a supply-chain surface.

Required controls:

- HTTPS source validation
- checksum verification
- version pinning
- on-disk manifest of installed asset metadata

Recommended asset manifest fields:

- asset name
- version
- source URL
- checksum
- installed at
- used by features

## Logging And Observability

Logs should support debugging without leaking too much.

## Minimum logs

- startup and shutdown
- config load result
- compatibility server enablement
- install/import decisions
- plugin launch failures
- tool execution summaries
- retry and compaction actions

## Redaction rules

Redact by default:

- secrets
- auth tokens
- prompt content when debug logging is off
- private filesystem paths in compatibility-facing error responses

## Metrics

Useful initial counters:

- runs started/completed/failed/cancelled
- provider request latency
- tool execution latency
- compaction count
- retry count
- compatibility request counts
- plugin crash count

## Upgrade Strategy

There are at least four upgradeable things:

- core binary
- managed assets
- imported skills
- imported plugins

They should not all share one opaque update mechanism.

Recommended commands:

- `matrixclaw self update`
- `matrixclaw asset update`
- `matrixclaw skill update <name>`
- `matrixclaw plugin update <name>`

## Backup And Recovery

At minimum, operators should be able to back up:

- config
- state database
- imported manifests
- exported sessions

Recommended recovery features:

- `matrixclaw export state`
- `matrixclaw import state`
- session export independent of plugin reinstall

## Failure Modes

### Startup failure

MatrixClaw should distinguish:

- config failure
- storage failure
- asset verification failure
- compatibility bind failure
- bridge failure

Startup should continue in degraded mode where safe.

Example:

- bridge runtime broken should not prevent plain chat usage

### Plugin failure

If a plugin crashes:

- record the failure
- surface clear operator diagnostics
- avoid taking down unrelated sessions where possible

### Provider failure

Provider failures should be classified and passed to retry policy rather than surfaced as untyped internal errors.

## Multi-User Considerations

Even if v1 is single-user first, the docs should not make later multi-user support impossible.

Prepare for:

- scoped API tokens
- per-user sessions
- per-user workspace ownership
- per-user plugin visibility

Do not assume global mutable state is always safe.

## Design Checks

The operational model is healthy if:

- default install is easy but not reckless
- imported artifacts always carry provenance
- bridge runtimes are visibly optional
- remote exposure is explicit, not accidental
- failures degrade functionality narrowly instead of crashing the whole system
