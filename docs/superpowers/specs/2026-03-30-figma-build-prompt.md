# Prompt For A Figma-Capable Agent

Create a new **Figma design file** named `MatrixClaw Product Shell`.

Add pages named:
- `Workspace`
- `Agents`
- `Agent Detail`
- `Skills`
- `MCP`
- `Gateway`

Then build two first-pass desktop application frames:

1. a tri-rail `Workspace` frame
2. an `Agent Detail` frame

## Product intent
MatrixClaw is a chat-first local-first agent runtime.

The UI must separate:
1. global capability catalogs: `Skills`, `MCP`, `Gateway`
2. per-agent bindings: which capabilities are enabled for a given agent
3. per-agent owned state: crown job, memory, messaging posture

## Architecture-sensitive rules
1. `Workspace` is for conversation and current run-state, not global configuration.
2. `Agent Detail` is for per-agent configuration.
3. `Skills`, `MCP`, and `Gateway` are global management pages.
4. Design assuming one workspace session is bound to one selected agent.
5. The transcript column must be visually dominant.

## Visual direction
Use a balanced dark product UI:
- calm, structured, not dashboard-heavy
- single cyan accent
- soft corners
- modern but operational

Suggested tokens:
- canvas `#070B14`
- surface-1 `#0D1422`
- surface-2 `#121B2C`
- surface-3 `#192538`
- primary text `#EAF0FA`
- secondary text `#B8C3D9`
- muted text `#8A96AE`
- border `#26344A`
- accent `#5BC0EB`

Suggested typography:
- display: `Space Grotesk`
- UI: `IBM Plex Sans`

## Workspace frame requirements
Desktop frame around `1440 x 1024`.

Use a three-column layout:
1. left rail: active agent summary, crown job summary, memory summary, enabled counts, quick links to agent/global pages
2. center column: chat header, transcript, composer
3. right rail: queue state, runtime posture, warnings, current run metadata

Do not put global settings forms into the workspace rail.

## Agent Detail frame requirements
Desktop frame around `1440 x 1024`.

Create a clean settings-style page with sections for:
1. identity
2. crown job
3. memory
4. enabled skills
5. enabled MCP servers
6. enabled gateways
7. agent messaging behavior

This page should feel like an agent cockpit, not a generic admin form.

## Additional direction
- reserve lighter placeholder coverage for the other pages
- prefer strong information hierarchy over too many equal-weight cards
- avoid the current UI's “everything at once” feeling
- make the architecture legible through layout

## Reference documents
Read and follow:
- `docs/superpowers/specs/2026-03-30-figma-design-handoff.md`
- `docs/superpowers/specs/2026-03-30-workspace-agent-configuration-design.md`

## Reference diagrams
Use these as architecture guides:
- Product architecture: https://www.figma.com/online-whiteboard/create-diagram/e6a4d960-2b1f-48cf-acb9-1cc712817158?utm_source=chatgpt&utm_content=edit_in_figjam&oai_id=&request_id=a2b9eb83-faff-425a-a13c-115e8204aedf
- Configuration ownership: https://www.figma.com/online-whiteboard/create-diagram/096ebc5c-9059-4ffd-a899-57dc979e956c?utm_source=chatgpt&utm_content=edit_in_figjam&oai_id=&request_id=8c1d7f43-3165-4f87-91b9-34fde3cc76b2
- Workspace run sequence: https://www.figma.com/online-whiteboard/create-diagram/ed31b9f6-9cad-484b-a181-03d2fc650bdc?utm_source=chatgpt&utm_content=edit_in_figjam&oai_id=&request_id=e0f29410-6743-44ba-bbbf-ba580086c637
