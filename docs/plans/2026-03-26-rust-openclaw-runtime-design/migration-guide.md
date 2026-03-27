# Migration Guide

## Purpose

This document explains how existing OpenClaw ecosystem artifacts map into MatrixClaw.

It is written for:

- users importing community assets
- maintainers deciding what to port first
- implementers building importers and diagnostics

## Migration Philosophy

MatrixClaw should not require users to understand internal Rust architecture to reuse common community assets.

Migration priority should be:

1. preserve useful text-first artifacts unchanged
2. adapt process-boundary plugins through shims
3. reject tightly coupled in-process extensions honestly

## Artifact Mapping

## 1. `SKILL.md` directories

OpenClaw shape:

```text
skills/coding-agent/
  SKILL.md
```

MatrixClaw mapping:

```text
~/.matrixclaw/skills/coding-agent/
  SKILL.md
  matrixclaw.skill.json
```

Migration behavior:

- preserve `SKILL.md`
- parse frontmatter if present
- normalize metadata into native manifest
- record provenance back to original source
- allow later agent-local enablement without mutating the imported package

Expected support:

- `native`

## 2. Plain markdown prompt bundles

OpenClaw or `pi`-style shape:

```text
some-package/
  prompts/
    review.md
    ship.md
```

MatrixClaw mapping:

- import as prompt resources or skill resources depending on structure
- optionally generate native metadata for discovery

Expected support:

- `native`

## 3. Workspace context files

Source shape:

```text
AGENTS.md
SOUL.md
USER.md
MEMORY.md
TOOLS.md
```

MatrixClaw mapping:

- load directly as workspace resources
- preserve contents verbatim
- document precedence and load order

Expected support:

- `native`

## 4. Process-boundary plugins

Source shape:

- subprocess tool
- JSON-RPC plugin
- MCP server

MatrixClaw mapping:

- install under `plugins/`
- generate `matrixclaw.plugin.json`
- launch via native adapter layer

Expected support:

- `shimmed` or `native`

## 5. OpenClaw extension packages with manifest and TypeScript runtime

Observed shape:

```text
extensions/anthropic/
  index.ts
  openclaw.plugin.json
  package.json
```

MatrixClaw migration paths:

### Path A: convert to process-boundary adapter

Use when:

- the extension can be represented as provider, tool, or channel behavior through a subprocess or MCP-style boundary

Expected support:

- `shimmed`

### Path B: optional bridge runtime

Use when:

- the extension can only run in Node/Bun but has a stable external contract

Expected support:

- `bridge_only`

### Path C: manual rewrite

Use when:

- the extension depends on direct in-process runtime APIs and lifecycle objects

Expected support:

- `unsupported`

## Example Migration Outcomes

## Example: importing `skills/coding-agent`

Input:

- `https://github.com/openclaw/openclaw/tree/main/skills/coding-agent`

Result:

- installs unchanged `SKILL.md`
- writes `matrixclaw.skill.json`
- registers as `native`

## Example: importing `extensions/anthropic`

Input:

- `https://github.com/openclaw/openclaw/tree/main/extensions/anthropic`

Likely result:

- detect `openclaw.plugin.json`
- detect TypeScript runtime files
- inspect whether behavior can be represented via provider adapter
- if yes, generate shimmed plugin manifest
- if no, mark `bridge_only` or `unsupported`

## Migration Assistant Behavior

MatrixClaw should provide clear operator messaging during migration.

## Native case

```text
Detected OpenClaw skill package
Class: skill_text
Tier: native
Action: importing unchanged
Next step: enable for an agent or select during setup
```

## Shimmed case

```text
Detected OpenClaw plugin package
Class: plugin_process
Tier: shimmed
Action: generating MatrixClaw adapter manifest
```

## Unsupported case

```text
Detected OpenClaw in-process extension
Class: plugin_inprocess
Tier: unsupported
Reason: requires direct OpenClaw runtime APIs
Suggestion: port to MCP, JSON-RPC, or CLI adapter
```

## Porting Guidance for Community Maintainers

## Best path for maintainers

If a community maintainer wants their OpenClaw package to work in MatrixClaw with minimal effort, the recommended order is:

1. publish skill/prompt assets as plain markdown resources
2. expose executable behavior through MCP or JSON-RPC subprocess boundaries
3. avoid requiring direct in-process runtime hooks
4. assume operators may install globally and enable per agent rather than dumping everything into one always-on prompt

## Anti-patterns to discourage

- extensions that require mutable access to runtime internals
- monkey-patching message/event objects
- hidden install-time Node/Bun assumptions
- undocumented package lifecycle side effects

## Compatibility Labels

Recommended labels for docs and CLI:

- `matrixclaw-native`
- `matrixclaw-shimmed`
- `matrixclaw-bridge`
- `matrixclaw-unsupported`

These labels should appear:

- during `compat inspect`
- during install
- in generated manifests
- in the compatibility registry

## Migration Backlog Priorities

Highest-value import targets:

1. top community `SKILL.md` packages
2. common workspace/context conventions
3. MCP-compatible tools
4. common provider/channel plugins that can be adapted through process boundaries
5. everything else

## Documentation Requirements

For every compatibility class, MatrixClaw should publish:

- what is supported
- how it is imported
- where it is installed
- how it is updated
- what breaks compatibility

## Open Questions

- whether MatrixClaw should support automatic conversion suggestions for `openclaw.plugin.json`
- whether a future compatibility linter should suggest MCP/JSON-RPC rewrites for unsupported packages
- whether bridge-only support should be hidden behind an experimental flag
