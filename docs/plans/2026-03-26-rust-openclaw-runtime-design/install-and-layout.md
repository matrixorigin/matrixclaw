# Install And Layout

## Purpose

This document defines where MatrixClaw installs itself, where imported ecosystem assets live, and how install/update/remove flows should behave.

The design goal is:

- no privileged writes by default
- predictable filesystem layout
- clear separation between runtime state and imported community artifacts

## Default Filesystem Layout

Suggested home directory:

- `~/.matrixclaw`

Suggested structure:

```text
~/.matrixclaw/
  bin/
    matrixclaw
  config/
    config.json
    users.json
  agents/
    default/
      agent.json
      enabled-skills.json
      channels.json
  state/
    compat-registry.json
    managed-assets.json
    sessions.db
  sessions/
    exports/
  skills/
    coding-agent/
      SKILL.md
      matrixclaw.skill.json
  plugins/
    anthropic/
      openclaw.plugin.json
      matrixclaw.plugin.json
      adapter/
  workspaces/
    default/
      AGENTS.md
      SOUL.md
      IDENTITY.md
      USER.md
      MEMORY.md
      TOOLS.md
      HEARTBEAT.md
  cache/
    downloads/
    git/
    bridge/
  assets/
    browser/
    stt/
  logs/
    matrixclaw.log
```

This layout deliberately separates:

- global installed assets
  - `skills/`, `plugins/`
- agent-local runtime metadata
  - `agents/<name>/enabled-skills.json`
- user-authored workspace context
  - `workspaces/<name>/...`

That keeps imported community packages immutable while still letting agents opt into different skill sets.

## Install Commands

## Binary installation

Recommended default:

```bash
curl -fsSL https://example.com/install.sh | sh
```

Installer behavior:

- installs binary to `~/.matrixclaw/bin/matrixclaw`
- updates shell profile only if needed
- does not require Bun, Node.js, or Docker
- does not require `sudo` by default

## Ecosystem installation

Recommended commands:

```bash
matrixclaw install <source>
matrixclaw skill install <source>
matrixclaw skill enable <name> --agent <agent>
matrixclaw skill disable <name> --agent <agent>
matrixclaw plugin install <source>
matrixclaw compat inspect <source>
matrixclaw update <name>
matrixclaw remove <name>
```

## Install flow details

### `matrixclaw skill install <source>`

Expected flow:

1. resolve source
2. inspect and classify
3. confirm support tier
4. copy or materialize artifact into `~/.matrixclaw/skills/<name>/`
5. write `matrixclaw.skill.json`
6. update central compatibility registry

### `matrixclaw skill enable <name> --agent <agent>`

Expected flow:

1. resolve installed skill from global store
2. resolve target agent metadata
3. record skill enablement under `agents/<agent>/enabled-skills.json`
4. make the skill available for explicit load during future runs
5. reflect the enabled state in the web UI and CLI

### `matrixclaw plugin install <source>`

Expected flow:

1. resolve source
2. inspect and classify
3. if `native` or `shimmed`, install into `~/.matrixclaw/plugins/<id>/`
4. if `bridge_only`, require explicit bridge enablement
5. if `unsupported`, stop with a precise diagnostic
6. write `matrixclaw.plugin.json`
7. update central compatibility registry

## Source resolution

Supported initial source forms:

- absolute or relative local path
- git URL
- GitHub URL
- archive URL

Recommended resolution cache:

- cloned repos under `~/.matrixclaw/cache/git`
- downloaded archives under `~/.matrixclaw/cache/downloads`

## Layout Rules

### Rule: imported artifacts are immutable inputs

Imported community files should be preserved as much as possible.

Allowed:

- adding companion manifests
- adding adapter files
- recording provenance

Avoid:

- rewriting imported files destructively
- silently flattening or renaming content without traceability

### Rule: runtime state stays out of imported artifact directories

Keep mutable runtime state under:

- `state/`
- `cache/`
- `logs/`

Do not pollute imported skill/plugin directories with volatile session state.

## Update Flow

## `matrixclaw update <name>`

Expected flow:

1. locate artifact in compatibility registry
2. resolve original source
3. fetch latest allowed revision
4. re-run compatibility inspection
5. if support tier is unchanged or improves, update in place
6. if support tier degrades, require explicit user confirmation
7. preserve prior manifest and provenance history

## Pinning

Recommended support:

- pin by git commit
- pin by tag
- pin by archive checksum

Store pins in provenance records.

## Removal Flow

## `matrixclaw remove <name>`

Expected behavior:

- remove installed artifact directory
- remove central registry entry
- preserve optional backup metadata for recovery
- refuse removal if another installed artifact depends on it unless forced

## Workspace Layout Strategy

MatrixClaw should distinguish:

- runtime home
  - `~/.matrixclaw`
- user workspaces
  - one or more agent workspaces under configured directories

Imported skills/plugins belong under the runtime home.
User-authored project files belong in configured workspaces.

Agent-local enablement metadata belongs under `agents/`, not in workspace files and not inside imported skill directories.

## UI Layout Implications

The filesystem layout should map cleanly to operator surfaces:

- setup wizard
  - initializes `config/`, creates the first `agents/<name>/`, and selects starter skills
- Skills page
  - reads from `skills/` plus compatibility registry state
- Plugins page
  - reads from `plugins/` plus compatibility registry state
- workspace/chat surface
  - reads from `workspaces/<name>/` and `agents/<name>/enabled-skills.json`

This is the key product lesson from FastClaw and PiClaw:

- skill and plugin inventory should be visible in management views
- file-centric work should happen in the workspace/chat surface
- the runtime home layout should support both without hidden coupling

## Managed Assets

Optional large assets belong under:

- `~/.matrixclaw/assets`

Examples:

- browser engine binaries
- OCR or STT models
- optional bridge runtimes

These assets must be:

- versioned
- checksum verified
- removable independently of the core binary

## Bridge Runtime Layout

If optional bridge support is enabled, isolate it under:

```text
~/.matrixclaw/bridge/
  runtimes/
  node_modules/
  adapters/
  logs/
```

Rules:

- bridge assets are not required for core runtime success
- bridge failures must not block MatrixClaw startup unless a bridge-only plugin is explicitly being activated

## Diagnostics and Logs

Suggested files:

- `logs/matrixclaw.log`
- `logs/install.log`
- `logs/compat-inspect.log`
- `logs/bridge.log`

Installer and import flows should produce concise user-facing output and detailed log files for failures.

## Example Install Outcomes

### Native skill install

Input:

- GitHub URL to `skills/coding-agent`

Output:

```text
Installed skill: coding-agent
Tier: native
Origin: openclaw
Path: ~/.matrixclaw/skills/coding-agent
```

### Shimmed plugin install

Input:

- OpenClaw plugin with process-boundary adapter path

Output:

```text
Installed plugin: anthropic
Tier: shimmed
Adapter: jsonrpc_stdio
Path: ~/.matrixclaw/plugins/anthropic
```

### Unsupported plugin install

Input:

- in-process TypeScript extension

Output:

```text
Install refused
Artifact: plugin_inprocess
Tier: unsupported
Reason: requires direct OpenClaw runtime APIs
Suggestion: port to MCP/JSON-RPC or enable a future bridge runtime
```

## Operational Policies

### Default no-sudo policy

All normal install and import flows should succeed without privileged writes.

### Atomic installs

Use temporary directories and rename-on-success for installs and updates.

### Crash-safe provenance writes

Update compatibility registry only after artifact materialization succeeds.

## Open Questions

- whether to allow project-local compatibility installs in addition to user-global installs
- whether the registry should track reverse dependencies between imported artifacts
- whether skills and plugins should share a single unified `install` namespace internally
