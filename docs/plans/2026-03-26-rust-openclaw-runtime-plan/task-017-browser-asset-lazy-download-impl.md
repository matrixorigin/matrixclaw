# Task 017: [IMPL] Browser asset lazy download

**depends-on**: task-017-browser-asset-lazy-download-test

## Description

Implement managed browser-asset lookup, first-use download, and cache reuse without introducing startup dependencies on that asset.

## Execution Context

**Task Number**: 017 of 019 (impl)  
**Phase**: Operations  
**Prerequisites**: The paired Red test fails because browser assets download at the wrong time or are not cached.

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

- Modify: `crates/app-host/src/assets.rs`
- Modify: `crates/manifests/src/config.rs`
- Create: `crates/app-host/src/asset_manifest.rs`
- Modify: `crates/app-host/src/main.rs`

## Steps

### Step 1: Re-run the failing test

- Confirm the lazy-download asset test still fails before implementation.

### Step 2: Implement minimal asset behavior

- Resolve managed asset presence from runtime storage.
- Trigger download on first browser-capability use only.
- Persist asset metadata and reuse cached assets on subsequent calls.

### Step 3: Verify Pass

- Run the targeted asset-lazy-download test and confirm it passes.

### Step 4: Regression sweep

- Re-run app-host tests to protect startup and setup behavior.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host browser_asset_lazy_download -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- Startup succeeds without browser assets installed.
- First use downloads and later uses reuse cached assets.
- The targeted scenario passes.
