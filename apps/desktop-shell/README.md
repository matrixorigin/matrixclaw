# MatrixClaw Desktop Shell Boundary

This directory defines the desktop shell boundary for MatrixClaw.

Current state:
- The browser-first UI boundary is owned by `app-host`.
- The desktop shell uses a Tauri 2 window with a product-style bootstrap surface before handing the same window to the loopback UI.
- The shell does not duplicate config, session, or execution logic.
- The current shell targets macOS first while keeping Linux and Windows as later wrappers over the same boundary.

Developer commands:

```bash
pnpm --dir apps/desktop-shell install
pnpm --dir apps/desktop-shell dev
pnpm --dir apps/desktop-shell build
pnpm --dir apps/desktop-shell test
```

Launch model:
- the Tauri host window loads `src/index.html`
- the bootstrap page polls `http://127.0.0.1:38495/healthz`
- when the runtime answers, the shell chooses `/setup` or `/workspace` and keeps startup inside the same window
- if the runtime is unavailable, the shell remains on a readable bootstrap state with retry instead of falling into a raw network failure

Preview and harness notes:
- append `?bootstrap-preview=1` while loading `src/index.html` to keep the shell on the bootstrap surface without automatic navigation
- `src/launcher.js` exports pure helpers for route resolution and startup-state rendering, covered by the local Node test
