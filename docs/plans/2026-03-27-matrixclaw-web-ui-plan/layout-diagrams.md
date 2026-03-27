# MatrixClaw Web UI Layout Diagrams

## Purpose

This document establishes the layout before implementation so the execution plan does not treat the UI as an abstract “web shell.”

The intended product shape is:
- browser-first
- workspace-first
- mobile-capable
- operator-friendly without collapsing into a generic admin dashboard

## Layout Principles

1. Setup is a focused onboarding flow, not the permanent app shell.
2. The main app is a workspace surface first, with management pages adjacent to it.
3. The left rail is for navigation and workspace structure.
4. The center is for conversation and composer.
5. The right rail is for context, queued state, and detail panels.
6. Mobile collapses rails into drawers and sheets, but preserves the same information model.
7. Execution provenance must be visible whenever code runs, especially for sandboxed execution.

## Route Map

```text
/
└── redirect
    ├── /setup        when config is missing
└── /workspace    when config exists

/setup
├── provider
├── workspace
├── auth
├── execution
└── review

/workspace
├── chat
├── files
├── skills
└── settings
```

Task 001 scaffold mapping:
- `ui/src/routes/+page.svelte` provides a developer landing surface for static preview builds
- `ui/src/routes/setup/+page.svelte` establishes the onboarding shell
- `ui/src/routes/workspace/+page.svelte` establishes the three-region workspace shell
- `crates/app-host/src/ui_assets.rs` defines the Rust-side source/build path contract used by later embedding work

## Information Architecture

```text
MatrixClaw UI
├── Setup Flow
│   ├── Provider selection
│   ├── Workspace root
│   ├── Auth token
│   └── Execution defaults
│
└── Main App Shell
    ├── Workspace / Chat
    │   ├── File explorer
    │   ├── Chat transcript
    │   ├── Composer
    │   ├── Queue state
    │   └── Execution provenance
    │
    ├── Skills
    │   ├── Installed inventory
    │   └── Agent-local enabled state
    │
    └── Settings
        ├── Provider summary
        ├── Workspace summary
        └── Execution backend policy
```

## Desktop Layout

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Top Bar                                                                     │
│ MatrixClaw | current agent | workspace | status | settings                  │
├───────────────┬──────────────────────────────────────────┬───────────────────┤
│ Left Rail     │ Main Surface                             │ Right Rail        │
│               │                                          │                   │
│ Nav           │ Chat transcript                          │ Run status        │
│ - Workspace   │ ┌──────────────────────────────────────┐ │ - active run     │
│ - Skills      │ │ assistant / tool / warning stream   │ │ - retry state    │
│ - Settings    │ │                                      │ │ - queue summary  │
│               │ └──────────────────────────────────────┘ │                   │
│ Files         │                                          │ Context panel     │
│ - tree        │ Composer                                 │ - selected file   │
│ - search      │ [ prompt box                        ]    │ - skill details   │
│ - reference   │ [ reference chips ] [ send ]            │ - env hints       │
│               │                                          │                   │
│               │ Execution badges                         │ Execution panel   │
│               │ - local                                 │ - backend used    │
│               │ - docker                                │ - priority order  │
│               │ - boxlite                               │ - fallback policy │
│               │                                          │ - timeout         │
│               │                                          │ - mounts / net    │
│               │                                          │ - stdout / stderr │
│               │                                          │ - unavailable err │
│               │                                          │                   │
├───────────────┴──────────────────────────────────────────┴───────────────────┤
│ Footer: local-only notice | asset download state | compat mode state        │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Mobile Layout

```text
┌──────────────────────────────┐
│ Top Bar                      │
│ Menu | MatrixClaw | Status   │
├──────────────────────────────┤
│ Main Surface                 │
│                              │
│ Chat transcript              │
│                              │
│ [ selected context chip ]    │
│                              │
├──────────────────────────────┤
│ Composer                     │
│ [ prompt input          ]    │
│ [ ref ] [ queue ] [ send ]   │
└──────────────────────────────┘

Left rail becomes:
- slide-out drawer for navigation and files

Right rail becomes:
- bottom sheet for queue state, execution details, file details, and skill details
```

## Setup Wizard Flow

```text
Start
  │
  ├─→ Provider step
  │     choose provider + model
  │
  ├─→ Workspace step
  │     choose workspace root + default agent name
  │
  ├─→ Auth step
  │     enter token / key material
  │
  ├─→ Execution step
  │     local vs sandbox default
  │     sandbox backend priority:
  │     1. docker
  │     2. boxlite
  │
  └─→ Review + save
        persist config
        persist execution defaults
        enter workspace shell
```

## Workspace Surface Detail

```text
Workspace page
├── Left rail
│   ├── workspace tree
│   ├── file search
│   └── reference action
│
├── Center column
│   ├── transcript stream
│   ├── queued steering markers
│   ├── queued follow-up markers
│   ├── execution backend badges on code/tool events
│   └── composer
│
└── Right rail
    ├── current run state
    ├── execution backend detail
    ├── selected file preview metadata
    └── enabled skills summary
```

## Skills Page Layout

```text
┌────────────────────────────────────────────────────┐
│ Skills                                             │
├──────────────────────┬─────────────────────────────┤
│ Installed skills     │ Selected skill detail       │
│ - imported/native    │ - provenance                │
│ - compatibility tier │ - description               │
│ - version            │ - enabled for current agent │
│                      │ - enable / disable action   │
└──────────────────────┴─────────────────────────────┘
```

## Tauri Shell Boundary

```text
┌──────────────────────────┐
│ Tauri shell              │
│ - native window chrome   │
│ - launch/attach logic    │
│ - loopback URL loading   │
└─────────────┬────────────┘
              │
              ▼
┌──────────────────────────┐
│ app-host                 │
│ - local HTTP surface     │
│ - setup API              │
│ - workspace API          │
│ - skills API             │
└─────────────┬────────────┘
              │
              ▼
┌──────────────────────────┐
│ SvelteKit static UI      │
│ - setup pages            │
│ - workspace shell        │
│ - skills/settings pages  │
└──────────────────────────┘
```

## Execution Backend Model

```text
Execution policy
├── local
│   └── direct host execution
│
└── sandboxed
    ├── docker   (priority 1)
    └── boxlite  (priority 2)
```

Rules:
- `docker` is the preferred sandbox backend
- `boxlite` is the secondary sandbox backend
- if sandboxing is required and neither backend is available, the UI must show a hard failure
- the UI must never silently imply local execution when a sandbox requirement failed
- transcript/tool UI should badge execution with `local`, `docker`, or `boxlite`

## Settings Page Execution Section

```text
Settings / Execution
├── Mode
│   ├── local
│   └── sandboxed
│
├── Sandbox backend priority
│   ├── docker
│   └── boxlite
│
├── Fallback policy
│   ├── require sandbox
│   └── allow fallback
│
└── Diagnostics
    ├── docker availability
    ├── boxlite availability
    └── last backend error
```

## Design Implications For Execution

- `task-001` must establish route structure and component boundaries, not just toolchain files.
- `task-003` and `task-004` should preserve the separation between setup flow and main app shell.
- `task-005`, `task-006`, and `task-007` must implement within the shell shown above instead of inventing ad hoc layouts.
- execution-aware UI must be treated as a first-class product surface
- sandbox backend naming must be explicit: `docker` first, `boxlite` second
- the Tauri shell task must keep the shell thin and must not duplicate app-host responsibilities.
