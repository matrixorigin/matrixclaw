# MatrixClaw Desktop Shell Boundary

This directory defines the optional desktop wrapper boundary for MatrixClaw.

Current state:
- The browser-first UI boundary is owned by `app-host`.
- The desktop shell is implemented as a thin Tauri 2 scaffold around the same loopback UI.
- The shell does not duplicate config, session, or execution logic.
- The current shell targets macOS first while keeping Linux and Windows as later wrappers over the same boundary.

Developer commands:

```bash
pnpm --dir apps/desktop-shell install
pnpm --dir apps/desktop-shell dev
pnpm --dir apps/desktop-shell build
```

Launch model:
- the Tauri host window loads `src/index.html`
- the bootstrap page redirects the webview to `http://127.0.0.1:38495/setup`
- later work can replace the static redirect with explicit attach-or-launch orchestration
