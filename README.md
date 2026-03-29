# MatrixClaw

MatrixClaw is a native Rust agent runtime that is designed to install as a single binary without requiring Bun, Node.js, or Docker for first-run setup.

The runtime stays Rust-only at execution time. The web UI uses `SvelteKit` as a build-time toolchain, but the shipped binary is expected to serve static assets directly without requiring a Node.js process on the target machine.

## Install

MatrixClaw is installed into a user-owned directory under `~/.matrixclaw/bin`.

## Version

```bash
matrixclaw version
```

## Web UI Workspace

The browser UI lives under [`ui/`](./ui) and is designed to build static assets for embedding into `app-host`.

Development commands:

```bash
bun install --cwd ui
bun run --cwd ui dev
bun run --cwd ui build
bun run --cwd ui check
```

Runtime note:

- `bun` and Node.js-compatible tooling are acceptable at build time for developers
- the installed MatrixClaw binary must not require a Node.js process at runtime

## Optional Desktop Shell

MatrixClaw also includes an optional macOS-first desktop wrapper scaffold under
[`apps/desktop-shell/`](./apps/desktop-shell). The desktop shell is intentionally thin:

- `app-host` remains the runtime owner
- the shell opens the same loopback UI boundary used by the browser flow
- Linux and Windows stay future targets through the same boundary
