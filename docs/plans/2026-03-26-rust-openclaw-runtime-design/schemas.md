# Schemas

## Purpose

This document defines the initial on-disk schemas and machine-readable outputs needed to make ecosystem compatibility real and testable.

These schemas are design targets, not final implementation constraints. Their purpose is to force clarity before code is written.

## Schema Design Principles

- prefer small, explicit manifests over implicit magic
- preserve original source metadata for imported OpenClaw assets
- separate origin metadata from normalized MatrixClaw runtime metadata
- make support tier and compatibility status explicit
- allow schema evolution with versioning from day one

## Common Enums

### Support tier

```json
["native", "shimmed", "bridge_only", "unsupported"]
```

### Artifact class

```json
[
  "skill_text",
  "workspace_convention",
  "plugin_process",
  "plugin_bridge",
  "plugin_inprocess"
]
```

### Source type

```json
["local_path", "git", "github", "archive", "registry"]
```

## `matrixclaw.skill.json`

Purpose:

- native normalized manifest for imported or first-party skills

Location:

- skill root directory

Versioning:

- required `schemaVersion`

### Required fields

- `schemaVersion`
- `name`
- `version`
- `description`
- `entry`
- `compat`
- `source`

### Optional fields

- `displayName`
- `tags`
- `requiresEnv`
- `runtimeHints`
- `activation`
- `provenance`

### Suggested schema

```json
{
  "schemaVersion": "1",
  "name": "coding-agent",
  "displayName": "Coding Agent",
  "version": "1.0.0",
  "description": "Coding workflow skill",
  "entry": "SKILL.md",
  "tags": ["coding", "assistant", "openclaw"],
  "requiresEnv": [],
  "runtimeHints": {
    "disableModelInvocation": false
  },
  "activation": {
    "installScope": "global",
    "defaultLoadMode": "on_demand"
  },
  "compat": {
    "origin": "openclaw",
    "artifactClass": "skill_text",
    "tier": "native",
    "importMode": "normalized"
  },
  "source": {
    "type": "github",
    "ref": "https://github.com/openclaw/openclaw",
    "revision": "main",
    "path": "skills/coding-agent"
  },
  "provenance": {
    "importedAt": "2026-03-26T20:00:00Z",
    "importedBy": "matrixclaw 0.1.0",
    "originalFiles": ["SKILL.md"]
  }
}
```

### Field notes

- `name`
  - normalized machine identifier
- `displayName`
  - optional UI label
- `entry`
  - relative path to the primary skill file
- `compat.origin`
  - where the artifact came from, such as `openclaw`, `pi`, or `matrixclaw`
- `compat.importMode`
  - `native`, `normalized`, `shimmed`, `bridge`
- `activation.installScope`
  - initial target is `global`
- `activation.defaultLoadMode`
  - `eager` or `on_demand`

## `enabled-skills.json`

Purpose:

- agent-local record of which installed skills are available to a specific agent

Location:

- `~/.matrixclaw/agents/<agent>/enabled-skills.json`

Suggested schema:

```json
{
  "schemaVersion": "1",
  "agent": "default",
  "skills": [
    {
      "name": "coding-agent",
      "source": "global_store",
      "loadMode": "on_demand"
    }
  ]
}
```

Design notes:

- this file should reference globally installed skills rather than copying them
- it is runtime metadata, not imported ecosystem content
- it supports the operator model of installed vs enabled vs loaded

## `matrixclaw.plugin.json`

Purpose:

- native normalized manifest for plugins and adapters

Location:

- plugin root directory

Versioning:

- required `schemaVersion`

### Required fields

- `schemaVersion`
- `id`
- `name`
- `kind`
- `compat`
- `source`

### Optional fields

- `displayName`
- `description`
- `entrypoint`
- `transport`
- `env`
- `permissions`
- `bridge`
- `capabilities`
- `provenance`

### Suggested schema

```json
{
  "schemaVersion": "1",
  "id": "anthropic",
  "name": "anthropic",
  "displayName": "Anthropic Provider",
  "description": "Provider plugin imported from OpenClaw",
  "kind": "provider",
  "entrypoint": {
    "command": "matrixclaw-plugin-adapter",
    "args": ["--manifest", "openclaw.plugin.json"]
  },
  "transport": {
    "type": "jsonrpc_stdio"
  },
  "env": {
    "required": ["ANTHROPIC_API_KEY"],
    "optional": []
  },
  "permissions": {
    "filesystem": "none",
    "network": "provider_only"
  },
  "capabilities": {
    "provides": ["provider"],
    "consumes": []
  },
  "compat": {
    "origin": "openclaw",
    "artifactClass": "plugin_process",
    "tier": "shimmed",
    "importMode": "adapter",
    "originalManifest": "openclaw.plugin.json"
  },
  "source": {
    "type": "github",
    "ref": "https://github.com/openclaw/openclaw",
    "revision": "main",
    "path": "extensions/anthropic"
  },
  "provenance": {
    "importedAt": "2026-03-26T20:00:00Z",
    "importedBy": "matrixclaw 0.1.0",
    "originalFiles": [
      "openclaw.plugin.json",
      "index.ts",
      "package.json"
    ]
  }
}
```

### Plugin kinds

Suggested initial values:

```json
["tool", "channel", "provider", "hook", "bridge_adapter"]
```

### Transport kinds

Suggested initial values:

```json
["jsonrpc_stdio", "mcp_stdio", "mcp_http", "bridge_runtime", "none"]
```

## `compat inspect` result

Purpose:

- machine-readable classifier output before install

Command:

```bash
matrixclaw compat inspect <source> --json
```

### Suggested schema

```json
{
  "schemaVersion": "1",
  "source": {
    "type": "github",
    "ref": "https://github.com/openclaw/openclaw",
    "path": "skills/coding-agent"
  },
  "detected": {
    "artifactClass": "skill_text",
    "tier": "native",
    "confidence": "high"
  },
  "signals": {
    "hasSkillMd": true,
    "hasOpenClawPluginManifest": false,
    "hasPackageJson": false,
    "hasTypeScriptEntrypoint": false,
    "hasProcessBoundaryMetadata": false
  },
  "decision": {
    "action": "install_native",
    "reason": "Directory contains SKILL.md and no executable runtime dependency markers"
  },
  "compat": {
    "origin": "openclaw",
    "importMode": "normalized"
  },
  "diagnostics": []
}
```

### Decision actions

Suggested initial values:

```json
[
  "install_native",
  "install_shim",
  "install_bridge",
  "reject_unsupported",
  "manual_review"
]
```

## Event envelope

Purpose:

- serialized event shape shared across persistence, UI streaming, and compatibility adapters

This does not require every event to be stored forever.
It defines the canonical wire shape for runtime events.

### Suggested schema

```json
{
  "schemaVersion": "1",
  "eventId": "evt_01HXYZ",
  "runId": "run_01HXYZ",
  "sessionId": "sess_01HXYZ",
  "turnId": "turn_01HXYZ",
  "sequence": 42,
  "timestamp": "2026-03-26T20:00:00Z",
  "type": "message_delta",
  "payload": {
    "messageId": "msg_01HXYZ",
    "delta": "hello"
  }
}
```

### Required fields

- `schemaVersion`
- `eventId`
- `runId`
- `sessionId`
- `sequence`
- `timestamp`
- `type`
- `payload`

### Notes

- `sequence` must be monotonic within a run
- `turnId` is required for turn-scoped events and optional otherwise
- `payload` should be event-specific and internally versioned through `schemaVersion`

## Session export schema

Purpose:

- portable durable transcript export
- backup and migration format

### Suggested schema

```json
{
  "schemaVersion": "1",
  "session": {
    "id": "sess_01HXYZ",
    "agentId": "default",
    "workspaceId": "default",
    "createdAt": "2026-03-26T20:00:00Z",
    "updatedAt": "2026-03-26T20:05:00Z"
  },
  "messages": [
    {
      "id": "msg_user_1",
      "role": "user",
      "kind": "user",
      "createdAt": "2026-03-26T20:00:00Z",
      "content": {
        "text": "build a Rust version"
      }
    },
    {
      "id": "msg_asst_1",
      "role": "assistant",
      "kind": "assistant",
      "createdAt": "2026-03-26T20:00:10Z",
      "content": {
        "text": "Here is the design."
      }
    }
  ],
  "compactions": [],
  "metadata": {
    "exportedAt": "2026-03-26T20:06:00Z",
    "exportedBy": "matrixclaw 0.1.0"
  }
}
```

### Export rules

- exports must reflect durable transcript state, not transient UI cache
- `kind` should preserve richer semantics than plain role names when needed
- tool calls and tool results should be exported as explicit message kinds

## Runtime config schema

Purpose:

- formalize the first-run config file written by setup

Suggested path:

- `~/.matrixclaw/config/config.json`

### Suggested schema

```json
{
  "schemaVersion": "1",
  "server": {
    "bind": "127.0.0.1",
    "port": 8342
  },
  "compat": {
    "openclaw": {
      "enabled": false,
      "bind": "127.0.0.1",
      "port": 8343
    }
  },
  "defaultAgent": {
    "workspaceId": "default",
    "provider": "openai_compatible",
    "model": "gpt-5.4"
  },
  "storage": {
    "sqlitePath": "~/.matrixclaw/state/sessions.db"
  },
  "execution": {
    "mode": "local"
  },
  "assets": {
    "autoDownload": true
  }
}
```

### Config validation rules

- bind addresses default to loopback
- compatibility server is disabled by default
- unknown top-level keys should be rejected with a clear message in v1 setup flows
- secret values should be referenced indirectly where possible

## Provenance record

Purpose:

- durable metadata for updates, removals, and audits

Location:

- central registry entry under MatrixClaw data directory

### Suggested schema

```json
{
  "schemaVersion": "1",
  "id": "skill:coding-agent",
  "type": "skill",
  "name": "coding-agent",
  "installedPath": "/home/user/.matrixclaw/skills/coding-agent",
  "artifactClass": "skill_text",
  "tier": "native",
  "origin": "openclaw",
  "source": {
    "type": "github",
    "ref": "https://github.com/openclaw/openclaw",
    "revision": "main",
    "path": "skills/coding-agent"
  },
  "importedByVersion": "0.1.0",
  "installedAt": "2026-03-26T20:00:00Z",
  "generatedFiles": [
    "matrixclaw.skill.json"
  ],
  "originalFiles": [
    "SKILL.md"
  ]
}
```

## Central registry file

Purpose:

- quick lookup for installed ecosystem artifacts

Suggested path:

- `~/.matrixclaw/state/compat-registry.json`

### Suggested top-level structure

```json
{
  "schemaVersion": "1",
  "artifacts": [
    {
      "id": "skill:coding-agent",
      "type": "skill",
      "name": "coding-agent",
      "artifactClass": "skill_text",
      "tier": "native"
    }
  ]
}
```

## Native runtime metadata vs imported metadata

MatrixClaw should preserve both:

- normalized native manifest for runtime use
- original imported metadata for provenance and re-import/update logic

Do not destructively rewrite imported manifests in place when avoidable.

## Validation Rules

### Skill validation

- `entry` must exist
- `name` must be normalized and unique within installed skill namespace
- `description` must be present
- imported artifact class must be `skill_text` or `workspace_convention`

### Plugin validation

- `kind` must be recognized
- if `transport.type` is process-boundary based, the corresponding entrypoint must be defined
- `bridge_runtime` transport requires explicit bridge support enabled
- plugin manifest must declare a support tier consistent with inspection results

### Event validation

- `eventId` must be unique
- `sequence` must be strictly increasing within a run
- `message_completed` payload must contain the exact finalized content persisted for that message id

### Config validation

- `server.port` and `compat.openclaw.port` must not collide unless explicitly supported later
- file paths must expand to user-writable locations by default
- execution mode must be one of `local`, `sandboxed`, or `disabled`

## Evolution Rules

- schema changes must bump `schemaVersion`
- unknown fields must be ignored, not rejected, where safe
- importers should preserve older manifests and write normalized companion files instead of destructive rewrites

## Open Questions

- whether `version` should be mandatory for local ad hoc skill directories
- whether provenance should live only in a central registry or also in each artifact manifest
- whether `compat inspect` should support signed compatibility policies in the future
- whether session export should embed queue state or keep exports transcript-only by default
