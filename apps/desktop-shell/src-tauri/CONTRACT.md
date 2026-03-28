# Desktop Shell Contract

The desktop shell must remain a product startup boundary around the MatrixClaw loopback UI surface.

Contract points:
- attach to the local `app-host` surface without spawning extra product windows
- keep startup in a single Tauri window and hand that same window to the loopback UI
- choose setup versus workspace intentionally from the runtime health contract
- surface startup failures as shell states instead of raw browser/network errors
- render the same web UI boundary used by browser flows once attach succeeds
- avoid duplicating config, session, or execution logic
- stay optional so the core runtime remains browser-first and independently testable

Current startup note:
- the shell now owns a bootstrap state machine in `src/launcher.js`
- the bootstrap reads `/healthz`, resolves `/setup` versus `/workspace`, and navigates the existing webview only after the runtime is reachable
- future work can add native app-host launch orchestration behind the same startup contract without moving runtime logic into the shell
