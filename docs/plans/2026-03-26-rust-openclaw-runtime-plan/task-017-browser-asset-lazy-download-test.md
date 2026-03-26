# Task 017: [TEST] Browser asset lazy download

**depends-on**: task-002-first-launch-setup-impl

## Description

Create a failing asset-management test proving browser-dependent capability downloads happen only on first use and do not block core runtime startup beforehand.

## Execution Context

**Task Number**: 017 of 019 (test)  
**Phase**: Operations  
**Prerequisites**: Runtime-home, config, and app-host startup behavior already exist.

## BDD Scenario

```gherkin
Scenario: Browser engine downloads only on first use
  Given MatrixClaw is installed without a browser engine asset
  When the user first invokes a browser-dependent capability
  Then MatrixClaw downloads the required asset into managed storage
  And subsequent browser requests reuse the installed asset
  And core chat and tool functionality still works without that asset before first use
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Create: `crates/app-host/tests/browser_asset_lazy_download.rs`
- Create: `crates/app-host/src/assets.rs`
- Modify: `crates/app-host/src/setup.rs`
- Modify: `crates/manifests/src/config.rs`

## Steps

### Step 1: Verify Scenario

- Confirm managed assets are optional, lazy, and versioned independently from the binary.

### Step 2: Create the failing Red test

- Start the runtime with no browser asset installed.
- Use a download service test double and a browser-capability stub.
- Assert that startup succeeds without the asset, first use downloads it, and second use reuses the cached asset.
- Keep the failure semantic by checking for eager download, repeated downloads, or blocked startup.

### Step 3: Lock the asset-manager contract

- Define asset metadata, downloader, and cache lookup interfaces needed by the test.
- Do not implement lazy-download behavior in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host browser_asset_lazy_download -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- One failing test covers lazy asset acquisition.
- The failure demonstrates incorrect asset timing or caching.
- Asset management remains isolated behind explicit interfaces and doubles.
