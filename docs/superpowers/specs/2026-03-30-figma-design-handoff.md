# MatrixClaw Figma Design Handoff

## Goal
Create a new Figma **design** file for the MatrixClaw product shell and build the first high-level application frames for:

1. `Workspace`
2. `Agents`
3. `Agent Detail`
4. `Skills`
5. `MCP`
6. `Gateway`

The first concrete frames to build are:

1. the tri-rail `Workspace` frame
2. the `Agent Detail` frame

This handoff is intended for a Figma-capable agent or application that has write access to the Figma design canvas.

## Product Model
MatrixClaw is a chat-first product shell around a local-first agent runtime.

The UI model should reflect three distinct layers:

1. `Global catalogs`
   Skills, MCP servers, and Messaging Gateways are managed globally.

2. `Per-agent bindings`
   Each agent enables a subset of the global skills, MCP servers, and gateways.

3. `Per-agent owned state`
   Crown job, memory, and agent-specific operating posture belong to the agent.

## Architectural Constraints
The design should make the runtime model legible instead of hiding it.

### Required separation
1. `Workspace`
   Daily-use surface for conversation and run-state only.

2. `Agent Detail`
   Per-agent configuration surface.

3. `Skills`, `MCP`, `Gateway`
   Global management pages, not embedded admin panels inside workspace.

### Runtime alignment
The UI should be designed assuming one of these architecture choices must be made:

1. `Preferred`: `session_id` implies a single bound `agent_id`
2. `Alternative`: `agent_id` is added explicitly to the live run request

For product simplicity, the design should assume the preferred model:

- one active workspace session belongs to one selected agent
- switching agents should feel like switching to that agent's session context
- agent config does not live inside the transcript rail

## Visual Direction
Use the approved system direction:

1. `Balanced Product`
   Dark, calm, structured, less noisy than the current dashboard UI.

2. `Single accent`
   Cyan/blue accent for primary actions and active states.

3. `Soft radius`
   Cards around 12, panels around 16, pills fully rounded.

## Suggested Token Foundation
Use these as starting values unless the destination design system already has equivalent tokens.

### Color
- `bg.canvas` `#070B14`
- `bg.surface.1` `#0D1422`
- `bg.surface.2` `#121B2C`
- `bg.surface.3` `#192538`
- `text.primary` `#EAF0FA`
- `text.secondary` `#B8C3D9`
- `text.muted` `#8A96AE`
- `border.default` `#26344A`
- `border.strong` `#33455F`
- `accent.primary` `#5BC0EB`
- `accent.hover` `#71CCF1`
- `accent.pressed` `#43B2DF`
- `accent.on` `#041018`
- `state.success` `#4FD1A5`
- `state.warning` `#F6C177`
- `state.error` `#F38BA8`

### Spacing
`4, 8, 12, 16, 20, 24, 32, 40, 48`

### Radius
- `input 10`
- `button 10`
- `card 12`
- `panel 16`
- `pill 999`

### Typography
Recommended families:

- `Display`: `Space Grotesk`
- `UI`: `IBM Plex Sans`

Suggested ramp:

- `12 / 16`
- `14 / 20`
- `16 / 24`
- `20 / 28`
- `24 / 32`

## Top-Level Information Architecture
Create these pages in the Figma file:

1. `Workspace`
2. `Agents`
3. `Agent Detail`
4. `Skills`
5. `MCP`
6. `Gateway`

## Frame Specs
### 1. Workspace
The workspace is a chat-first tri-rail application frame.

#### Desktop frame
- Width: `1440`
- Height: `1024`
- Desktop app window style
- Dark product shell, not a marketing page

#### Layout
1. `Left rail`
   Width around `280-320`
   Purpose: selected agent summary and quick navigation outward

2. `Center column`
   Flexible and dominant visual weight
   Purpose: transcript + composer

3. `Right rail`
   Width around `300-340`
   Purpose: run-state, queue, warnings, execution posture

#### Left rail content
Top to bottom:

1. product header / route nav
2. active agent card
3. crown job summary block
4. memory summary block
5. enabled capability counts
   - skills
   - MCP
   - gateways
6. quick links
   - open agent detail
   - go to skills
   - go to MCP
   - go to gateway

Do **not** put global configuration forms directly in this rail.

#### Center column content
1. chat header
   - selected agent name
   - concise crown job subtitle
   - session status chip

2. transcript panel
   - assistant and user turns
   - attached context chips
   - lightweight inline tool/run indicators only

3. composer dock
   - prompt field
   - attach context
   - send / run

This column must clearly dominate the page.

#### Right rail content
1. queue state card
2. execution posture card
3. runtime warnings / policy card
4. current run metadata

Do **not** put permanent settings here.

### 2. Agent Detail
This is the per-agent cockpit.

#### Desktop frame
- Width: `1440`
- Height: `1024`

#### Layout
Two-column page with a strong primary content area.

1. `Left navigation / header column`
   Narrow summary rail or page-level nav support

2. `Main content area`
   Sectioned settings page

#### Sections
Create visible sections for:

1. `Identity`
   - agent name
   - description / role summary
   - status

2. `Crown Job`
   - concise editable role contract
   - should feel like a brief, not a giant raw prompt editor

3. `Memory`
   - memory summary
   - recent memory signals
   - controls to inspect / prune / pin / configure

4. `Enabled Skills`
   - rows sourced from global skills catalog
   - enablement state per agent
   - health/availability cues

5. `Enabled MCP Servers`
   - rows sourced from global MCP catalog
   - enablement state per agent
   - health/availability cues

6. `Enabled Gateways`
   - rows sourced from global gateway catalog
   - enablement state per agent
   - health/availability cues

7. `Agent Messaging Behavior`
   - delivery / routing posture where relevant

## Supporting Page Direction
These pages do not need full detail in the first pass, but the file should reserve them clearly.

### Agents
Agent directory page.

Each row/card should show:
- agent name
- crown job summary
- memory status
- enabled capability counts
- last active
- open / switch actions

### Skills
Global skills catalog.

Each row should show:
- global definition
- install / health state
- version or source
- enabled by N agents

### MCP
Global MCP server registry.

Each row should show:
- server name
- connection / health state
- enabled by N agents
- routing / availability cues if relevant

### Gateway
Global messaging gateway registry.

Each row should show:
- gateway name
- ingress / egress health
- enabled by N agents
- routing status

## Component Inventory
Build or represent these components at least at a first-pass fidelity level.

1. app shell header
2. route navigation item
3. page title / subtitle block
4. agent summary card
5. section card / panel
6. status badge
7. capability count pill
8. transcript message block
9. composer input area
10. runtime state card
11. settings row with toggle / status / metadata
12. empty-state / helper copy block

## Interaction Intent To Preserve In The Mockups
The frames should visually imply these behaviors:

1. changing the selected agent updates workspace context
2. workspace is for talking to the agent, not configuring the system
3. agent detail is where per-agent configuration lives
4. skills / MCP / gateway pages are global, with per-agent enablement happening elsewhere

## Things To Avoid
1. Do not recreate the current dashboard-heavy, many-boxes-at-once feel.
2. Do not mix global catalog management directly into workspace.
3. Do not make the right rail as visually heavy as the center conversation rail.
4. Do not turn crown job into a giant raw prompt textarea in the first frame.
5. Do not present session diagnostics as if they were durable agent configuration.

## Recommended Build Order In Figma
1. Create the design file
2. Add pages: `Workspace`, `Agents`, `Agent Detail`, `Skills`, `MCP`, `Gateway`
3. Build a cover/overview region if desired
4. Build the desktop `Workspace` frame first
5. Build the desktop `Agent Detail` frame second
6. Add lighter placeholder frames for `Agents`, `Skills`, `MCP`, and `Gateway`
7. Refine hierarchy, spacing, and contrast only after the architecture is visually legible

## Source References
Primary spec:
- [2026-03-30-workspace-agent-configuration-design.md](/Users/randomradio/src/matrixclaw/docs/superpowers/specs/2026-03-30-workspace-agent-configuration-design.md)

Related architecture references:
- [rust-openclaw-runtime architecture](/Users/randomradio/src/matrixclaw/docs/plans/2026-03-26-rust-openclaw-runtime-design/architecture.md)
- [matrix gateway architecture](/Users/randomradio/src/matrixclaw/docs/plans/2026-03-28-matrix-gateway-design/architecture.md)
- [node architecture](/Users/randomradio/src/matrixclaw/docs/plans/2026-03-28-node-design/architecture.md)

## Existing FigJam Diagrams
Use these as architecture references while building the design file:

1. Product architecture
   https://www.figma.com/online-whiteboard/create-diagram/e6a4d960-2b1f-48cf-acb9-1cc712817158?utm_source=chatgpt&utm_content=edit_in_figjam&oai_id=&request_id=a2b9eb83-faff-425a-a13c-115e8204aedf

2. Configuration ownership
   https://www.figma.com/online-whiteboard/create-diagram/096ebc5c-9059-4ffd-a899-57dc979e956c?utm_source=chatgpt&utm_content=edit_in_figjam&oai_id=&request_id=8c1d7f43-3165-4f87-91b9-34fde3cc76b2

3. Workspace run sequence
   https://www.figma.com/online-whiteboard/create-diagram/ed31b9f6-9cad-484b-a181-03d2fc650bdc?utm_source=chatgpt&utm_content=edit_in_figjam&oai_id=&request_id=e0f29410-6743-44ba-bbbf-ba580086c637
