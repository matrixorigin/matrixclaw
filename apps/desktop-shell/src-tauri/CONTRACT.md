# Desktop Shell Contract

The optional desktop shell must remain a thin wrapper around the MatrixClaw loopback UI boundary.

Contract points:
- launch or attach to the local `app-host` surface
- render the same web UI boundary used by browser flows
- avoid duplicating config, session, or execution logic
- stay optional so the core runtime remains browser-first

Current scaffold note:
- the Tauri shell scaffold now exists in this directory
- the shell still stays intentionally thin and points at the loopback UI boundary
- future work can add explicit app-host launch-or-attach orchestration without moving runtime logic into the shell
