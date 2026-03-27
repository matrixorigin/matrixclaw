# MatrixClaw Web UI Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Load `superpowers:executing-plans` skill using the Skill tool to implement this plan task-by-task.

**Goal:** Ship a browser-first MatrixClaw UI using `SvelteKit`, embedded into `app-host`, with an optional macOS-first `Tauri 2` shell boundary that can later expand to Linux and Windows.

**Architecture:** Keep the Rust runtime as the source of truth and treat the UI as static web assets served by `app-host` over loopback. Build the UI in `ui/` with `SvelteKit`, embed its production assets into `app-host`, and add a thin `Tauri` shell only after the browser-first flow works cleanly. This preserves the self-hosted binary model while keeping the UX aligned with FastClaw and PiClaw lessons. Execution provenance and sandbox policy must be explicit in the UI, with `docker` as sandbox priority 1 and `boxlite` as priority 2.

**Tech Stack:** Rust workspace, `app-host`, `SvelteKit`, `pnpm`, `Vitest`, optional `Tauri 2`

**Design Support:**
- [BDD Specs](../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md)
- [Architecture](../2026-03-26-rust-openclaw-runtime-design/architecture.md)
- [Install And Layout](../2026-03-26-rust-openclaw-runtime-design/install-and-layout.md)
- [Delivery Plan](../2026-03-26-rust-openclaw-runtime-design/delivery-plan.md)
- [Layout Diagrams](./layout-diagrams.md)

## Context

MatrixClaw currently has the runtime, manifests, compatibility boundary, and install/bootstrap path, but it does not yet have a real UI. The binary only writes first-run config and execution defaults, which is enough for tests but not enough for the product direction already captured in the design docs. The next step should not be “add some screens”; it should establish a clean UI architecture that preserves the binary-first runtime, supports browser-first setup, and leaves room for a macOS-first desktop wrapper without forcing that wrapper to own the runtime.

This plan is intentionally exclusive to the UI slice. It does not re-plan the already-completed runtime, compatibility, or manifest work. It focuses on the browser-first web surface and the shell boundary around it.

| Aspect | Current State | Target State |
|--------|--------------|--------------|
| App entrypoint | [`app-host`](/home/momo/src/matrixclaw/crates/app-host/src/lib.rs) only supports `version` and silent first-launch config bootstrap | `app-host` starts a local HTTP surface that serves setup and workspace UI |
| Setup experience | [`setup.rs`](/home/momo/src/matrixclaw/crates/app-host/src/setup.rs) writes config directly with no operator flow | browser-first setup wizard validates and persists provider, workspace, auth, and execution defaults |
| UI assets | no frontend workspace, no static asset pipeline, no embedded UI shell | `ui/` builds static assets that are embedded and served by the Rust binary |
| Workspace UX | no file explorer, no chat/workspace surface, no queue controls | workspace-first surface with explorer, file reference insertion, and steering/follow-up controls |
| Skill/operator UX | install flows exist in Rust, but no management views | Skills inventory and agent-local enablement visible in the UI |
| Execution visibility | sandbox and execution policy live mostly as backend/runtime concerns | setup, settings, transcript events, and detail panels expose `local`, `docker`, and `boxlite` execution clearly |
| Desktop packaging | no native shell | optional `Tauri 2` shell for macOS first, with Linux/Windows expansion later |

## Layout First

Execution should treat layout as a design constraint, not as implementation fallout.

The diagrams in [layout-diagrams.md](/home/momo/src/matrixclaw/docs/plans/2026-03-27-matrixclaw-web-ui-plan/layout-diagrams.md) establish:
- route map for setup vs main app shell
- desktop three-column workspace layout
- mobile collapse strategy
- Skills page split view
- execution backend visibility model
- Tauri shell boundary

Top-level shell model:

```text
left rail      center column                right rail
-----------    -------------------------    -------------------
nav + files    transcript + composer        queue + details
```

This means:
- setup work must not be planned as generic forms without a route flow
- workspace work must include composer placement and file rail behavior
- skills work must assume a management page, not a modal bolted onto chat
- execution backend choice and failure state must be visible in the product, not buried in config files

## Execution Plan

```yaml
tasks:
  - id: "001"
    subject: "Set up SvelteKit UI workspace and embedding contracts"
    slug: "set-up-sveltekit-ui-workspace-and-embedding-contracts"
    type: "setup"
    depends-on: []
  - id: "002-test"
    subject: "Write embedded asset pipeline test"
    slug: "write-embedded-asset-pipeline-test"
    type: "test"
    depends-on: ["001"]
  - id: "002-impl"
    subject: "Implement embedded asset pipeline"
    slug: "implement-embedded-asset-pipeline"
    type: "impl"
    depends-on: ["002-test"]
  - id: "003-test"
    subject: "Write local setup server contract test"
    slug: "write-local-setup-server-contract-test"
    type: "test"
    depends-on: ["002-impl"]
  - id: "003-impl"
    subject: "Implement local setup server and shell routing"
    slug: "implement-local-setup-server-and-shell-routing"
    type: "impl"
    depends-on: ["003-test"]
  - id: "004-test"
    subject: "Write setup wizard persistence test"
    slug: "write-setup-wizard-persistence-test"
    type: "test"
    depends-on: ["003-impl"]
  - id: "004-impl"
    subject: "Implement setup wizard persistence flow"
    slug: "implement-setup-wizard-persistence-flow"
    type: "impl"
    depends-on: ["004-test"]
  - id: "005-test"
    subject: "Write workspace explorer and file reference test"
    slug: "write-workspace-explorer-and-file-reference-test"
    type: "test"
    depends-on: ["003-impl"]
  - id: "005-impl"
    subject: "Implement workspace explorer and file reference flow"
    slug: "implement-workspace-explorer-and-file-reference-flow"
    type: "impl"
    depends-on: ["005-test"]
  - id: "006-test"
    subject: "Write queued steering controls UI test"
    slug: "write-queued-steering-controls-ui-test"
    type: "test"
    depends-on: ["003-impl"]
  - id: "006-impl"
    subject: "Implement queued steering controls UI"
    slug: "implement-queued-steering-controls-ui"
    type: "impl"
    depends-on: ["006-test"]
  - id: "007-test"
    subject: "Write skills inventory and enablement test"
    slug: "write-skills-inventory-and-enablement-test"
    type: "test"
    depends-on: ["003-impl"]
  - id: "007-impl"
    subject: "Implement skills inventory and enablement UI"
    slug: "implement-skills-inventory-and-enablement-ui"
    type: "impl"
    depends-on: ["007-test"]
  - id: "008-test"
    subject: "Write execution backend visibility test"
    slug: "write-execution-backend-visibility-test"
    type: "test"
    depends-on: ["003-impl"]
  - id: "008-impl"
    subject: "Implement execution backend visibility UI"
    slug: "implement-execution-backend-visibility-ui"
    type: "impl"
    depends-on: ["008-test"]
  - id: "009-test"
    subject: "Write Tauri shell boundary test"
    slug: "write-tauri-shell-boundary-test"
    type: "test"
    depends-on: ["003-impl"]
  - id: "009-impl"
    subject: "Implement macOS-first Tauri shell scaffold"
    slug: "implement-macos-first-tauri-shell-scaffold"
    type: "impl"
    depends-on: ["009-test"]
```

**Task File References (for detailed BDD scenarios):**
- [Task 001: Set up SvelteKit UI workspace and embedding contracts](./task-001-set-up-sveltekit-ui-workspace-and-embedding-contracts-setup.md)
- [Task 002: Write embedded asset pipeline test](./task-002-write-embedded-asset-pipeline-test-test.md)
- [Task 002: Implement embedded asset pipeline](./task-002-implement-embedded-asset-pipeline-impl.md)
- [Task 003: Write local setup server contract test](./task-003-write-local-setup-server-contract-test-test.md)
- [Task 003: Implement local setup server and shell routing](./task-003-implement-local-setup-server-and-shell-routing-impl.md)
- [Task 004: Write setup wizard persistence test](./task-004-write-setup-wizard-persistence-test-test.md)
- [Task 004: Implement setup wizard persistence flow](./task-004-implement-setup-wizard-persistence-flow-impl.md)
- [Task 005: Write workspace explorer and file reference test](./task-005-write-workspace-explorer-and-file-reference-test-test.md)
- [Task 005: Implement workspace explorer and file reference flow](./task-005-implement-workspace-explorer-and-file-reference-flow-impl.md)
- [Task 006: Write queued steering controls UI test](./task-006-write-queued-steering-controls-ui-test-test.md)
- [Task 006: Implement queued steering controls UI](./task-006-implement-queued-steering-controls-ui-impl.md)
- [Task 007: Write skills inventory and enablement test](./task-007-write-skills-inventory-and-enablement-test-test.md)
- [Task 007: Implement skills inventory and enablement UI](./task-007-implement-skills-inventory-and-enablement-ui-impl.md)
- [Task 008: Write execution backend visibility test](./task-008-write-execution-backend-visibility-test-test.md)
- [Task 008: Implement execution backend visibility UI](./task-008-implement-execution-backend-visibility-ui-impl.md)
- [Task 009: Write Tauri shell boundary test](./task-009-write-tauri-shell-boundary-test-test.md)
- [Task 009: Implement macOS-first Tauri shell scaffold](./task-009-implement-macos-first-tauri-shell-scaffold-impl.md)

## BDD Coverage

Design-rooted scenarios status:
- `User installs MatrixClaw without privileged writes` → already implemented in the runtime baseline, not re-planned here
- `First launch opens setup without prior manual configuration` → covered by tasks `003`, `004`
- `Final assistant answer is generated once` → already implemented in the runtime baseline, not re-planned here
- `Tool calls extend the turn loop` → already implemented in the runtime baseline, not re-planned here
- `Tool validation can block unsafe execution` → already implemented in the runtime baseline, not re-planned here
- `Steering message is delivered before the next assistant turn` → runtime behavior already implemented; UI exposure covered by task `006`
- `Follow-up message is delivered only after the current run completes` → runtime behavior already implemented; UI exposure covered by task `006`
- `Stored transcript matches visible behavior` → already implemented in the runtime baseline, not re-planned here
- `Session resumes after restart` → already implemented in the runtime baseline, not re-planned here
- `Runtime compacts context before retrying an overflowed request` → already implemented in the runtime baseline, not re-planned here
- `Compaction preserves role semantics` → already implemented in the runtime baseline, not re-planned here
- `OpenClaw-compatible client lists agents` → already implemented in the runtime baseline, not re-planned here
- `OpenClaw-compatible chat request reaches the internal runtime` → already implemented in the runtime baseline, not re-planned here
- `User installs a text skill originally built for OpenClaw` → runtime behavior already implemented; UI exposure covered by task `007`
- `User installs a subprocess-compatible plugin originally built for OpenClaw` → already implemented in the runtime baseline, not re-planned here
- `User tries to install an in-process OpenClaw extension tied to JS internals` → already implemented in the runtime baseline, not re-planned here
- `Browser engine downloads only on first use` → already implemented in the runtime baseline, not re-planned here
- `Safe local execution works without Docker` → already implemented in the runtime baseline, not re-planned here
- `Optional sandbox mode is enabled explicitly` → already implemented in the runtime baseline, not re-planned here

Scope-specific UI scenarios derived from the existing architecture and delivery docs:
- `Embedded web UI shell is served from app-host` → tasks `001`, `002`, `003`
- `Workspace explorer inserts file references without mutating files` → tasks `005`
- `Queued steering and follow-up controls are visible and correct in the local UI` → task `006`
- `Skills inventory reflects installed and enabled state separately` → task `007`
- `Execution surface reflects local vs docker vs boxlite backend` → task `008`
- `Desktop shell launches the same loopback UI boundary` → task `009`

This plan is intentionally scoped to the web UI and shell layer. The coverage table above distinguishes already-landed runtime scenarios from the new UI slice so execution can stay focused without pretending the underlying runtime work still needs planning.

## Dependency Chain

```text
task-001 (ui workspace setup)
    │
    └─→ task-002-test → task-002-impl
                         │
                         └─→ task-003-test → task-003-impl
                                      │
                                      ├─→ task-004-test → task-004-impl
                                      ├─→ task-005-test → task-005-impl
                                      ├─→ task-006-test → task-006-impl
                                      ├─→ task-007-test → task-007-impl
                                      ├─→ task-008-test → task-008-impl
                                      └─→ task-009-test → task-009-impl
```

**Analysis**:
- No circular dependencies are intended.
- Logical flow is foundation first: UI workspace → embedded asset pipeline → local setup/web shell → feature surfaces.
- After `task-003-impl`, feature work should fan out in parallel because explorer, queue controls, skills inventory, execution visibility, and shell boundary are independent slices.
- Every implementation task depends only on its paired Red task.

---

## Execution Handoff

**Plan complete and saved to `docs/plans/2026-03-27-matrixclaw-web-ui-plan/`. Execution options:**

**1. Orchestrated Execution (Recommended)** - Load `superpowers:executing-plans` skill using the Skill tool.

**2. Direct Agent Team** - Load `superpowers:agent-team-driven-development` skill using the Skill tool.

**3. BDD-Focused Execution** - Load `superpowers:behavior-driven-development` skill using the Skill tool for specific scenarios.
