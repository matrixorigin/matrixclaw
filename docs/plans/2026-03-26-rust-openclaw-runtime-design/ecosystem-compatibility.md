# Ecosystem Compatibility

## Goal

Adopt as much of the existing OpenClaw community ecosystem as possible without compromising the internal Rust architecture.

The compatibility design must optimize for three outcomes:

1. users can install common community skills with minimal or no changes
2. subprocess-style plugins can be adapted through stable boundaries
3. unsupported artifacts fail with precise diagnostics instead of vague incompatibility

It must also optimize for operator ergonomics:

4. users can see what is installed, enabled, and loadable without reading the filesystem by hand
5. agent-local skill behavior is explicit rather than hidden inside one global skill pile

## Observed Ecosystem Shape

Based on upstream references:

- `pi` and related ecosystems already treat many skills as filesystem resources
  - `SKILL.md` directories
  - markdown prompt assets
  - package-managed resource bundles
- OpenClaw has a large `skills/` tree that strongly suggests community skill reuse is already organized around skill directories containing `SKILL.md`
- OpenClaw also has `extensions/` packages that include TypeScript code and plugin metadata such as `openclaw.plugin.json`

This implies that “OpenClaw ecosystem compatibility” is really two different product problems:

- compatibility for data-like prompt/skill assets
- compatibility for executable extension/plugin packages

Those should be handled differently.

## Compatibility Model

## Axis 1: Artifact class

Artifacts should be classified into one of these classes at install time:

- `skill_text`
  - markdown skills, `SKILL.md`, prompt bundles
- `workspace_convention`
  - `AGENTS.md`, `SOUL.md`, `MEMORY.md`, `USER.md`, `TOOLS.md`, related support files
- `plugin_process`
  - subprocess plugins, JSON-RPC plugins, CLI tools, MCP servers
- `plugin_bridge`
  - JS/TS plugins that can run in a separate runtime through a bridge
- `plugin_inprocess`
  - extensions that expect direct in-process access to OpenClaw or `pi` runtime internals

## Axis 2: Support tier

Each artifact class maps to a support tier:

- `native`
  - install and run directly in MatrixClaw
- `shimmed`
  - install with an adapter layer
- `bridge_only`
  - requires optional external runtime support
- `unsupported`
  - cannot run meaningfully without a rewrite

## Proposed matrix

| Artifact class | Expected source examples | Support tier | Notes |
|---|---|---|---|
| `skill_text` | `skills/<name>/SKILL.md`, plain markdown skill repos | `native` | Highest-priority compatibility target |
| `workspace_convention` | `AGENTS.md`, `SOUL.md`, `MEMORY.md`, `USER.md`, `TOOLS.md` | `native` | Load and preserve semantics where possible |
| `plugin_process` | MCP servers, JSON-RPC subprocess plugins, CLI tool wrappers | `shimmed` or `native` | Prefer process protocols over runtime embedding |
| `plugin_bridge` | Node/Bun packages with stable external bridge contract | `bridge_only` | Optional, not part of core runtime guarantee |
| `plugin_inprocess` | TS extensions tightly coupled to OpenClaw/`pi` internals | `unsupported` | Should fail clearly |

## Installation Pipeline

## Command model

Recommended install commands:

```bash
matrixclaw install <source>
matrixclaw skill install <source>
matrixclaw plugin install <source>
matrixclaw compat inspect <source>
```

## Install flow

1. resolve source
   - local path
   - git URL
   - archive URL
   - future registry identifier
2. inspect artifact
   - directory structure
   - metadata files
   - runtime files
3. classify artifact
   - artifact class
   - support tier
4. choose install strategy
   - native import
   - shim adapter
   - bridge-required
   - reject
5. record provenance
   - source URL/path
   - detected type
   - installed version or revision
   - support tier
   - imported files

## Example outcomes

### Native skill import

Input:

- `skills/coding-agent/SKILL.md`

Result:

- installed into MatrixClaw skill store
- indexed as a native skill
- available immediately with no Node/Bun requirement

### Shimmed plugin import

Input:

- subprocess JSON-RPC plugin repo

Result:

- installed into managed plugin directory
- wrapped with MatrixClaw adapter metadata
- launched as an external process by the runtime

### Unsupported in-process extension

Input:

- OpenClaw TypeScript extension expecting direct access to OpenClaw internals

Result:

- installer stops
- reports exact reason
- may optionally suggest bridge mode or rewrite guide

## Skill Compatibility Design

The main lesson from FastClaw is that skill compatibility is not only an import problem. It is also an operator workflow problem.

MatrixClaw should distinguish three different skill lifecycle stages:

1. install into the global skill store
2. enable for one or more agents
3. load into an active run when needed

Those stages should be visible in both CLI and web UI.

## Native skill package format

MatrixClaw should support a native manifest while still importing OpenClaw-style skills.

Recommended native metadata file:

- `matrixclaw.skill.json`

Minimal fields:

- `name`
- `version`
- `description`
- `entry`
- `compat`
- `source`

Example:

```json
{
  "name": "coding-agent",
  "version": "1.0.0",
  "description": "Coding workflow skill",
  "entry": "SKILL.md",
  "compat": {
    "origin": "openclaw",
    "tier": "native"
  },
  "source": {
    "type": "git",
    "ref": "https://github.com/openclaw/openclaw"
  }
}
```

## Importing OpenClaw-style skills

Import rules:

- if a directory contains `SKILL.md`, treat it as a skill root
- preserve frontmatter when present
- normalize metadata into `matrixclaw.skill.json`
- preserve original source path and revision
- validate naming and description constraints

This follows the same broad direction already used by `pi`-style resource loading.

## Skill activation model

Recommended behavior:

- `matrixclaw skill install <source>`
  - imports the skill into the global runtime-managed store
- `matrixclaw skill enable <skill> --agent <agent>`
  - marks the skill as available to a specific agent or workspace
- `matrixclaw skill disable <skill> --agent <agent>`
  - removes the skill from that agent's enabled set
- `load_skill`
  - runtime tool that loads one of the already-enabled skills into the active run context on demand

Design rules:

- installed is not the same as enabled
- enabled is not the same as eagerly loaded
- a skill should not need to be reparsed from the original source tree every turn
- agent-local enablement should be stored as runtime metadata, not by mutating imported skill packages

This combines FastClaw's operator-facing skill model with OpenClaw ecosystem import goals.

## Operator surfaces for skills

MatrixClaw should expose skills in three places:

- setup wizard
  - pick starter skills for the first agent
- Skills page
  - inspect installed skills, provenance, support tier, and update status
- Agent settings or workspace panel
  - enable and disable skills for a specific agent

The chat/workspace surface should also support:

- seeing which skills are enabled for the current agent
- loading an enabled skill during a run
- attaching workspace files into the prompt alongside skill context

## Workspace convention compatibility

MatrixClaw should natively support common context file conventions:

- `AGENTS.md`
- `CLAUDE.md`
- `SOUL.md`
- `IDENTITY.md`
- `USER.md`
- `MEMORY.md`
- `TOOLS.md`
- `HEARTBEAT.md`

Support policy:

- load them as first-class runtime resources
- preserve user-authored content verbatim
- avoid silently rewriting semantics
- record precedence and load order

## Plugin Compatibility Design

## Preferred plugin target

The preferred long-term plugin contract should be process-boundary based:

- MCP
- JSON-RPC over stdio
- stable CLI bridge contracts

This is the compatibility sweet spot:

- reusable by existing communities
- language-agnostic
- safe for Rust core architecture
- easier to version and test

The main lesson from FastClaw here is product visibility:

- plugins should have a first-class operator page
- compatibility state should be visible before activation
- unsupported artifacts should show exact reason codes and guidance instead of generic install failure

## OpenClaw plugin manifest adaptation

If an artifact contains metadata such as `openclaw.plugin.json`, MatrixClaw should:

- parse and retain the original manifest
- classify runtime assumptions
- map supported fields into a native manifest such as `matrixclaw.plugin.json`
- refuse unsupported lifecycle hooks explicitly

Suggested native plugin manifest fields:

- `id`
- `name`
- `kind`
- `entrypoint`
- `transport`
- `requires`
- `compat`
- `env`
- `permissions`

## Bridge runtime policy

Optional bridge support may exist for some JS/Bun plugins, but it must be clearly isolated.

Rules:

- bridge support is optional
- bridge support must never be required for basic MatrixClaw installation
- bridge-managed plugins must be labeled as `bridge_only`
- bridge failures must not compromise the native runtime

## Compatibility Inspection

The runtime should expose inspection before install:

```bash
matrixclaw compat inspect <source>
```

Recommended output:

- detected artifact type
- support tier
- required runtime
- native/shim/bridge status
- reasons for any unsupported classification
- suggested migration path

Example:

```text
Source: github.com/example/openclaw-awesome-plugin
Detected: plugin_inprocess
Tier: unsupported
Reason: requires in-process TypeScript extension API
Suggestion: use bridge mode or port to MCP/JSON-RPC
```

## Migration Paths

Every installed or inspected artifact should fall into one of these operator-visible paths:

### Works unchanged

- native markdown skills
- standard workspace files

### Works with generated shim

- subprocess plugins
- MCP-compatible tools
- JSON-RPC plugins with stable contracts

### Requires optional bridge runtime

- some JS/Bun plugins with externalized contracts

### Requires manual port

- in-process extensions tied to OpenClaw internals

## Registry and Provenance

## Supported sources

Initial sources:

- local path
- git URL
- GitHub repo URL
- archive URL

Registry support can come later.

## Provenance recording

For every imported artifact, store:

- original source
- resolved revision or checksum
- detected artifact class
- support tier
- imported file list
- generated shim or manifest paths
- install timestamp

This is important for updates, debugging, and safe removal.

## Testing Strategy

## Fixture categories

- native text skill fixture
- OpenClaw `SKILL.md` directory fixture
- subprocess plugin fixture
- MCP server fixture
- bridge-only plugin fixture
- unsupported in-process extension fixture

## Required tests

- classifier returns the correct artifact class
- installer produces the correct native manifest
- shimmed plugins launch through the adapter successfully
- unsupported artifacts fail with precise diagnostics
- provenance is written correctly
- imported skills remain usable after restart

## Design Decisions

### Decision: prioritize text skill compatibility first

Chosen because it delivers the highest ecosystem reuse for the lowest architectural cost.

### Decision: prefer process-boundary plugins over in-process extension emulation

Chosen because it preserves Rust core integrity and creates a better long-term extension model.

### Decision: classify before install

Chosen because users need to know whether an artifact is native, shimmed, bridged, or unsupported before they commit to it.

### Decision: compatibility claims must be tiered and explicit

Chosen because “supports OpenClaw plugins” is misleading unless the support class is visible and enforced.
