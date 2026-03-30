# MatrixClaw Workspace And Agent Configuration Design

## Summary

MatrixClaw should move from a mixed dashboard workspace toward a chat-first product shell with clear separation between conversation, runtime posture, global capability management, and per-agent configuration.

The workspace becomes the place where a user talks to the currently selected agent. Global systems such as Skills, MCP servers, and Messaging Gateway are managed on their own dedicated pages. Per-agent decisions such as crown job, memory, and enabled bindings live on a dedicated agent detail page.

## Problem

The current UI is confusing because multiple mental models compete on the same screen:

- The workspace tries to be chat surface, file browser, queue console, execution console, and configuration center at the same time.
- Agent identity is weak, so users cannot easily tell who they are talking to or what that agent is configured to do.
- Global capabilities and per-agent enablement are not cleanly separated.
- Long-lived configuration sits too close to ephemeral runtime status, which makes the product feel noisy and difficult to scan.

## Design Goals

- Make `/workspace` feel decisively chat-first.
- Make the current agent explicit at all times.
- Separate global capability management from per-agent binding.
- Keep runtime and queue feedback visible without turning workspace into an admin console.
- Give memory and crown job first-class treatment as agent-owned configuration.
- Preserve a stable product shell and predictable navigation between operational and configuration surfaces.

## Non-Goals

- This design does not define backend schema changes.
- This design does not redesign onboarding in detail.
- This design does not collapse Skills, MCP, and Gateway into a single capabilities hub.
- This design does not introduce multi-agent orchestration inside one workspace screen.

## Product Model

MatrixClaw has two configuration levels:

### Global

Global pages define and manage reusable capabilities:

- Skills catalog
- MCP server registry
- Messaging Gateway connections

Global pages own installation, connection status, health, and shared definitions.

### Per-Agent

Each agent chooses how it uses those global capabilities:

- enabled skills
- enabled MCP servers
- enabled gateways
- crown job
- memory
- agent-specific messaging behavior

Per-agent pages own bindings and agent-local behavior. They do not redefine global systems.

## Information Architecture

### Primary Routes

- `/workspace`: chat-first operational surface for the selected agent
- `/agents`: agent directory and agent switching surface
- `/agents/:agentId`: per-agent configuration cockpit
- `/skills`: global skills catalog management
- `/mcp`: global MCP server management
- `/gateway`: global messaging gateway management
- `/setup`: installation and initial runtime configuration

### Navigation Rules

- The app shell top-level navigation remains stable across routes.
- From `/workspace`, clicking the active agent opens `/agents/:agentId`.
- From `/workspace`, clicking Skills, MCP, or Gateway summaries opens the corresponding global page.
- Returning to `/workspace` restores the previously active agent context.

## Workspace Design

### Mental Model

`/workspace` has one primary job: talk to the selected agent with clear context.

### Layout

The workspace uses a three-column layout:

- Left rail: agent control summary
- Center column: conversation
- Right rail: runtime state

### Left Rail: Agent Control Summary

The left rail shows compact, high-signal information about the currently selected agent:

- active agent switcher
- crown job summary
- memory summary
- enabled counts for skills, MCP servers, and gateways
- links to `/agents/:agentId`, `/skills`, `/mcp`, and `/gateway`

The left rail does not host heavy editing forms. It is a summary and navigation surface.

### Center Column: Conversation

The center column owns the main workflow:

- transcript
- composer
- attached reference chips
- current agent identity
- minimal run/session metadata

The center column should always be visually dominant. It must read as the main action surface.

### Right Rail: Runtime State

The right rail is reserved for operational status tied to the current run:

- queue state
- execution posture
- backend visibility
- policy warnings
- temporary run controls such as queue actions

This rail should not become a home for long-term configuration.

## Agents Directory

`/agents` is the directory of available agents.

Each agent row or card should show:

- name
- crown job
- memory state
- enabled skills count
- enabled MCP count
- enabled gateway count
- last active state
- quick actions to switch or open details

This page answers the question: which agent do I want to work with?

## Agent Detail Page

`/agents/:agentId` is the per-agent cockpit.

### Responsibilities

- edit crown job
- inspect and manage memory
- enable or disable skills from the global skills catalog
- enable or disable MCP servers from the global MCP registry
- enable or disable gateways from the global gateway registry
- define agent-specific messaging behavior

### Rules

- This page can only change per-agent bindings and agent-owned state.
- It cannot install a new skill, connect a new MCP server, or create a new gateway definition.
- Each binding row should show global health and local enablement together.

## Global Pages

### `/skills`

Owns the global catalog of skills:

- installed skills
- skill health or compatibility posture
- usage count across agents

### `/mcp`

Owns the global MCP registry:

- connected servers
- connection health
- availability
- usage count across agents

### `/gateway`

Owns global messaging gateway setup:

- available channels
- connection state
- health
- usage count across agents

### Shared Global Page Rules

- Global pages manage definitions, status, and health.
- Global pages do not silently alter per-agent enablement.
- Global pages should show downstream impact, such as how many agents depend on each item.

## Interaction Rules

### Agent Switching

When a user switches agents from `/workspace`:

- the left rail summary updates
- the center chat header updates
- composer context updates
- the active session context updates to the selected agent

The change should feel immediate and explicit.

### Workspace To Config Navigation

- Clicking the agent summary opens `/agents/:agentId`.
- Clicking Skills, MCP, or Gateway summaries opens the relevant global page.
- Returning to workspace restores the same active agent.

### Global vs Per-Agent Editing

- Editing `/skills`, `/mcp`, or `/gateway` changes global definitions only.
- Editing `/agents/:agentId` changes only that agent's bindings and local state.

### Memory

- Memory is owned per agent.
- Workspace shows only summary and recent signal.
- Full memory management lives on `/agents/:agentId`.

### Crown Job

- Crown job is owned per agent.
- Workspace shows a short summary.
- Full editing lives on `/agents/:agentId`.

## Design System Direction

The visual direction is `Balanced Product` with a single accent and soft radii.

### Color Tokens

- `bg.canvas`: `#070B14`
- `bg.surface.1`: `#0D1422`
- `bg.surface.2`: `#121B2C`
- `bg.surface.3`: `#192538`
- `text.primary`: `#EAF0FA`
- `text.secondary`: `#B8C3D9`
- `text.muted`: `#8A96AE`
- `border.default`: `#26344A`
- `border.strong`: `#33455F`
- `accent.primary`: `#5BC0EB`
- `accent.hover`: `#71CCF1`
- `accent.pressed`: `#43B2DF`
- `accent.on`: `#041018`
- `state.success`: `#4FD1A5`
- `state.warning`: `#F6C177`
- `state.error`: `#F38BA8`
- `focus.ring`: `#5BC0EB`

### Spacing Scale

- `4`
- `8`
- `12`
- `16`
- `20`
- `24`
- `32`
- `40`
- `48`

### Radius Scale

- input: `10`
- button: `10`
- card: `12`
- panel: `16`
- pill: `999`

### Type System

- display font: `Space Grotesk`
- UI font: `IBM Plex Sans`
- sizes:
  - `12 / 16`
  - `14 / 20`
  - `16 / 24`
  - `20 / 28`
  - `24 / 32`
- weights:
  - `400`
  - `500`
  - `600`
  - `700`

## Component Contracts

### Agent Summary Card

Used in workspace left rail and agent directory.

Shows:

- name
- crown job summary
- memory status
- enabled capability counts

### Capability Binding List

Used on `/agents/:agentId`.

Each row shows:

- global item name
- health
- enabled state for this agent
- any local override

### Global Catalog Row

Used on `/skills`, `/mcp`, and `/gateway`.

Each row shows:

- definition or connection state
- health
- usage count across agents

### Memory Block

Shows summary first and supports deeper management from the agent page.

### Crown Job Block

Shows a short role contract and edit affordance without becoming a raw prompt dump.

## Implementation Notes

- Route ownership should align with the new IA rather than forcing everything into `/workspace`.
- Workspace should consume compact summary projections from agent/global state rather than rendering full configuration editors inline.
- The current shell navigation should expand to support the new routes without changing the chat-first emphasis of `/workspace`.

## Verification Strategy

The implementation plan should verify:

- workspace remains chat-first at desktop widths
- active agent context survives navigation
- global edits do not silently change per-agent bindings
- per-agent edits do not mutate global definitions
- memory and crown job are visible in workspace and editable on `/agents/:agentId`
- separate Skills, MCP, and Gateway pages are reachable and coherent

## Open Questions Resolved

- Workspace is chat-first.
- Global capabilities are on separate pages, not a combined hub.
- Per-agent enablement lives on a dedicated agent page, not in a workspace drawer or modal.
- Skills, MCP, and Gateway are globally managed and selectively enabled per agent.
