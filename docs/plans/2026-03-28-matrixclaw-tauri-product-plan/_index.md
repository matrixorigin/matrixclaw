# MatrixClaw Tauri Product Plan

**Goal:** Turn MatrixClaw into a real single-window macOS Tauri app by bundling the UI and runtime into a self-contained product, replacing the preview-style shell with a desktop-grade app shell, and proving packaged first-run behavior.

**Architecture:** Tauri becomes the primary product boundary. `app-host` remains the runtime core and local service boundary, but the product no longer depends on repo-relative asset discovery or browser-first redirect flows. The UI remains SvelteKit, but it is restructured into a persistent app shell with nested onboarding and workspace surfaces.

**Design Support:**
- [Long-Horizon Prompt](../../long-horizon/Prompt.md)
- [Long-Horizon Plans](../../long-horizon/Plans.md)
- [Web UI Layout Diagrams](../2026-03-27-matrixclaw-web-ui-plan/layout-diagrams.md)
- [Install And Layout](../2026-03-26-rust-openclaw-runtime-design/install-and-layout.md)
- [Delivery Plan](../2026-03-26-rust-openclaw-runtime-design/delivery-plan.md)

**Research Inputs:**
- Current packaging failure comes from repo-relative UI asset discovery in [crates/app-host/src/ui_assets.rs](/home/momo/src/matrixclaw/crates/app-host/src/ui_assets.rs) and binary-only installation in [scripts/install.sh](/home/momo/src/matrixclaw/scripts/install.sh).
- Current desktop shell remains a redirect wrapper in [apps/desktop-shell/src/launcher.js](/home/momo/src/matrixclaw/apps/desktop-shell/src/launcher.js) and does not yet define real product chrome.
- Tauri bundle guidance supports a first-class macOS `.app` boundary and bundling custom files into the app package: <https://v2.tauri.app/distribute/macos-application-bundle/>
- Tauri’s SvelteKit guidance supports keeping the frontend static and bundled for desktop delivery: <https://v2.tauri.app/start/frontend/sveltekit/>
- Apple’s macOS Human Interface Guidelines remain the UX reference for windows, toolbars, sidebars, and search-driven navigation: <https://developer.apple.com/design/human-interface-guidelines/windows>

## Product Principles

1. The first serious MatrixClaw release is an app, not a wrapped website.
2. The app opens into one single window only.
3. The app shell is persistent; setup, workspace, skills, and settings are sections inside the same product.
4. The packaged app must not require a source checkout or repo-relative files.
5. Product verification must test packaged launch, first-run setup, and real runtime behavior.

## Execution Plan

```yaml
tasks:
  - id: "001-test"
    subject: "Write bundled asset packaging test"
    slug: "write-bundled-asset-packaging-test"
    type: "test"
    depends-on: []
  - id: "001-impl"
    subject: "Implement bundled asset packaging"
    slug: "implement-bundled-asset-packaging"
    type: "impl"
    depends-on: ["task-001-write-bundled-asset-packaging-test-test"]
  - id: "002-test"
    subject: "Write tauri embedded runtime startup test"
    slug: "write-tauri-embedded-runtime-startup-test"
    type: "test"
    depends-on: ["task-001-implement-bundled-asset-packaging-impl"]
  - id: "002-impl"
    subject: "Implement tauri embedded runtime startup"
    slug: "implement-tauri-embedded-runtime-startup"
    type: "impl"
    depends-on: ["task-002-write-tauri-embedded-runtime-startup-test-test"]
  - id: "003-test"
    subject: "Write desktop app shell contract test"
    slug: "write-desktop-app-shell-contract-test"
    type: "test"
    depends-on: ["task-001-implement-bundled-asset-packaging-impl"]
  - id: "003-impl"
    subject: "Implement desktop app shell architecture"
    slug: "implement-desktop-app-shell-architecture"
    type: "impl"
    depends-on: ["task-003-write-desktop-app-shell-contract-test-test"]
  - id: "004-test"
    subject: "Write multi-step onboarding flow test"
    slug: "write-multi-step-onboarding-flow-test"
    type: "test"
    depends-on: ["task-003-implement-desktop-app-shell-architecture-impl"]
  - id: "004-impl"
    subject: "Implement multi-step onboarding flow"
    slug: "implement-multi-step-onboarding-flow"
    type: "impl"
    depends-on: ["task-004-write-multi-step-onboarding-flow-test-test"]
  - id: "005-test"
    subject: "Write desktop workspace pane decomposition test"
    slug: "write-desktop-workspace-pane-decomposition-test"
    type: "test"
    depends-on: ["task-003-implement-desktop-app-shell-architecture-impl"]
  - id: "005-impl"
    subject: "Implement desktop workspace pane decomposition"
    slug: "implement-desktop-workspace-pane-decomposition"
    type: "impl"
    depends-on: ["task-005-write-desktop-workspace-pane-decomposition-test-test"]
  - id: "006-test"
    subject: "Write packaged product verification test"
    slug: "write-packaged-product-verification-test"
    type: "test"
    depends-on: ["task-002-implement-tauri-embedded-runtime-startup-impl", "task-004-implement-multi-step-onboarding-flow-impl", "task-005-implement-desktop-workspace-pane-decomposition-impl"]
  - id: "006-impl"
    subject: "Implement packaged product verification"
    slug: "implement-packaged-product-verification"
    type: "impl"
    depends-on: ["task-006-write-packaged-product-verification-test-test"]
```

## Task File References

- [Task 001 Test: Write bundled asset packaging test](./task-001-write-bundled-asset-packaging-test-test.md)
- [Task 001 Impl: Implement bundled asset packaging](./task-001-implement-bundled-asset-packaging-impl.md)
- [Task 002 Test: Write tauri embedded runtime startup test](./task-002-write-tauri-embedded-runtime-startup-test-test.md)
- [Task 002 Impl: Implement tauri embedded runtime startup](./task-002-implement-tauri-embedded-runtime-startup-impl.md)
- [Task 003 Test: Write desktop app shell contract test](./task-003-write-desktop-app-shell-contract-test-test.md)
- [Task 003 Impl: Implement desktop app shell architecture](./task-003-implement-desktop-app-shell-architecture-impl.md)
- [Task 004 Test: Write multi-step onboarding flow test](./task-004-write-multi-step-onboarding-flow-test-test.md)
- [Task 004 Impl: Implement multi-step onboarding flow](./task-004-implement-multi-step-onboarding-flow-impl.md)
- [Task 005 Test: Write desktop workspace pane decomposition test](./task-005-write-desktop-workspace-pane-decomposition-test-test.md)
- [Task 005 Impl: Implement desktop workspace pane decomposition](./task-005-implement-desktop-workspace-pane-decomposition-impl.md)
- [Task 006 Test: Write packaged product verification test](./task-006-write-packaged-product-verification-test-test.md)
- [Task 006 Impl: Implement packaged product verification](./task-006-implement-packaged-product-verification-impl.md)

## BDD Coverage

- `Packaged app launches without repo-relative assets` -> task pair `001`
- `Tauri owns single-window startup and runtime attachment` -> task pair `002`
- `Desktop shell feels like an app rather than a preview site` -> task pair `003`
- `First-run setup behaves like a real onboarding flow` -> task pair `004`
- `Workspace feels like a desktop tool with persistent panes` -> task pair `005`
- `Packaged product verification catches regressions before release` -> task pair `006`

## Dependency Chain

```text
task-001-test -> task-001-impl -> task-002-test -> task-002-impl
                     |
                     +--------> task-003-test -> task-003-impl
                                              |              |
                                              v              v
                                        task-004-test   task-005-test
                                              |              |
                                              v              v
                                        task-004-impl   task-005-impl
                                              \             /
                                               \           /
                                                v         v
                                           task-006-test -> task-006-impl
```

## Agent Team Execution Model

Use an agent team for implementation because the milestone has real parallel lanes with different owning files.

Suggested lanes:

- **Architect / Integrator**
  - owns product-boundary decisions, long-horizon sync, final integration
  - owns `docs/long-horizon/`, plan updates, `apps/desktop-shell/src-tauri/`, and shared packaging decisions
- **Packaging Implementer**
  - owns bundled asset embedding and packaged startup
  - primary files: `crates/app-host/src/ui_assets.rs`, `crates/app-host/src/server.rs`, `apps/desktop-shell/src-tauri/`
- **Shell/UI Implementer**
  - owns persistent app shell and workspace pane decomposition
  - primary files: `ui/src/routes/+layout.svelte`, `ui/src/routes/workspace/`, shared UI shell components
- **Onboarding Implementer**
  - owns multi-step setup flow and setup state model
  - primary files: `ui/src/routes/setup/`, `ui/src/lib/setup/`, setup-related tests
- **Reviewer / Verifier**
  - owns packaged product smoke harnesses and regression checks
  - primary files: `scripts/verify-*`, Playwright/Tauri smoke tests, product install tests

Parallel batches:

1. `001` then `002` on the packaging/runtime lane
2. after `001`, run `003` in parallel with `002`
3. after `003`, run `004` and `005` in parallel
4. finish with `006` as the integration and release gate

## Verification Gates

Focused checks:
- `cargo test -p matrixclaw-app-host`
- `pnpm --dir ui check`
- `pnpm --dir ui build`
- product-specific packaging and launch tests added by this milestone

Full regression gate:
- `cargo fmt --all --check`
- `cargo test -p matrixclaw-app-host`
- `cargo test -p matrixclaw-compat-openclaw`
- `pnpm --dir ui check`
- `pnpm --dir ui build`
- `pnpm --dir ui test:e2e`
- `./scripts/verify-live-runtime.sh`
- `./scripts/verify-served-transports.sh`
- `./scripts/verify-matrix-gateway.sh`
- new packaged-product verification script from task pair `006`

## Execution Handoff

This plan is the milestone execution artifact for Milestone 03 in `docs/long-horizon/Plans.md`.
Execute it before expanding Node work or external connector scope further.
