# MatrixClaw Product Shell V2 From Figma Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild the MatrixClaw shell to match the new Figma light design system and product-shell layouts, while adding the missing agent, MCP, and gateway surfaces needed to make the design real.

**Architecture:** Keep `session-runtime` pure and session-driven. Add file-backed agent profiles, global catalog snapshots, and a central `session_id -> agent_id` binding in `app-host`, then rebuild the SvelteKit route family around the Figma shell (`/workspace`, `/agents`, `/agents/:agentId`, `/skills`, `/mcp`, `/gateway`). Preserve the existing loopback/Tauri boundary and drive the work with Rust contract tests plus Playwright UI tests.

**Tech Stack:** SvelteKit 2, TypeScript, Playwright, Rust `app-host`, Rust `session-runtime`, Tauri desktop shell, file-backed JSON manifests under runtime home.

---

## Design Sources

Use these as the visual source of truth for the implementation.

- Design system: `https://www.figma.com/design/saWzuqSrtD1xWLaAQPO1Y8/MatrixClaw-%E2%80%94-Light-Design-System?m=auto&t=xnE1GQ4F44g5k30s-6`
- Product shell: `https://www.figma.com/design/BoF8jPIAD922rYDV0aWk0g/MatrixClaw-%E2%80%94-Product-Shell-v2?m=auto&t=xnE1GQ4F44g5k30s-6`

Important implementation note:
- The newer Figma design system is light and supersedes the earlier dark token appendix in [2026-03-30-workspace-agent-configuration-design.md](/Users/randomradio/src/matrixclaw/docs/superpowers/specs/2026-03-30-workspace-agent-configuration-design.md).
- Keep the information architecture from the spec, but use the Figma files as the visual source of truth.

## Scope Check

This plan covers one coherent subsystem: the product shell plus the minimal backend surface required to support it.

Included:
- shared shell/navigation update
- workspace redesign
- agents directory and agent detail
- global `skills`, `mcp`, and `gateway` pages
- agent/session binding needed to make the UI truthful

Not included:
- onboarding redesign beyond preserving compatibility with the new shell
- runtime-core memory semantics beyond summary/projection data needed by the UI
- new external gateway transports beyond surfacing current state

## File Structure Map

### Existing files to modify
- `ui/src/lib/app-shell/state.ts`
  - expand route metadata and route labels to cover the full product shell
- `ui/src/routes/+layout.svelte`
  - apply shared shell tokens and top navigation aligned with the Figma shell
- `ui/src/routes/workspace/+page.svelte`
  - replace the current explorer-heavy pane composition with the Figma tri-rail workspace
- `ui/src/routes/skills/+page.svelte`
  - convert from mixed global/per-agent toggle page into a global catalog page
- `ui/src/lib/workspace/shell.ts`
  - replace current queue/execution card helpers with the Figma run-state cards
- `ui/src/lib/http.ts`
  - keep shared fetch helpers, extend only if typed helpers reduce route duplication
- `crates/app-host/src/http/mod.rs`
  - register new HTTP routes for agents, MCP, gateway, and binding-backed agent runs
- `crates/app-host/src/http/agent_api.rs`
  - extend run payload parsing so the shell can resolve/bind selected agents for sessions
- `crates/app-host/src/http/skills_api.rs`
  - keep per-agent enablement but add a global catalog view with enabled-by counts
- `crates/app-host/src/server.rs`
  - update seeded runtime fixture data if needed for richer shell smoke coverage

### New frontend files to create
- `ui/src/lib/theme/product-shell.css`
  - shared Figma-derived CSS variables and reusable shell surface classes
- `ui/src/lib/agents/index.ts`
  - typed agent directory/detail API contracts and helpers
- `ui/src/lib/agents/session.ts`
  - selected-agent/session coordination helpers for the workspace shell
- `ui/src/lib/catalogs/mcp.ts`
  - typed MCP catalog contract for the global page and agent detail page
- `ui/src/lib/catalogs/gateway.ts`
  - typed gateway catalog contract for the global page and agent detail page
- `ui/src/routes/agents/+page.svelte`
  - agents directory page
- `ui/src/routes/agents/[agentId]/+page.svelte`
  - agent detail page
- `ui/src/routes/mcp/+page.svelte`
  - global MCP catalog page
- `ui/src/routes/gateway/+page.svelte`
  - global gateway catalog page
- `ui/tests/agents_directory.spec.ts`
  - Playwright contract for the new `/agents` route
- `ui/tests/agent_detail.spec.ts`
  - Playwright contract for the new `/agents/:agentId` route
- `ui/tests/global_catalog_pages.spec.ts`
  - Playwright coverage for `/skills`, `/mcp`, and `/gateway`
- `ui/tests/workspace_tri_rail.spec.ts`
  - Playwright coverage for the redesigned workspace shell

### New backend files to create
- `crates/app-host/src/agent_store.rs`
  - file-backed agent profiles and per-agent binding summaries
- `crates/app-host/src/session_binding_store.rs`
  - file-backed `session_id -> agent_name` mapping with drift protection
- `crates/app-host/src/http/agents_api.rs`
  - HTTP contract for agent directory and agent detail
- `crates/app-host/src/http/mcp_api.rs`
  - HTTP contract for global MCP catalog
- `crates/app-host/src/http/gateway_api.rs`
  - HTTP contract for global gateway catalog
- `crates/app-host/tests/agent_directory_contract.rs`
  - Rust contract test for listing agents and reading agent detail
- `crates/app-host/tests/skills_catalog_contract.rs`
  - Rust contract test for global skills catalog counts
- `crates/app-host/tests/mcp_catalog_contract.rs`
  - Rust contract test for MCP catalog loading and fallback defaults
- `crates/app-host/tests/gateway_catalog_contract.rs`
  - Rust contract test for gateway catalog loading and fallback defaults
- `crates/app-host/tests/session_agent_binding.rs`
  - Rust contract test for stable session-to-agent binding

## Architectural Decisions To Lock Before Coding

1. `session_id` implies one bound `agent_id`
   - the UI sends the selected agent on the first run for a session
   - `app-host` persists `session_id -> agent_name`
   - subsequent queue reads and streaming runs resolve agent context from the binding store

2. Agent-owned data stays outside `session-runtime`
   - `session-runtime` continues to own transcript, queue, and compaction only
   - `app-host` owns agent profile lookup, global catalog lookup, and projection onto runtime-facing requests

3. The current low-complexity HTTP router remains flat
   - prefer `/api/agents/detail?agent=atlas` over adding a new path-parameter parser to the handwritten router
   - keep the UI route tree expressive, but keep backend route parsing simple and testable

4. Global pages are read-mostly in the first pass
   - `skills` keeps one existing mutation: enable/disable via agent detail bindings
   - `mcp` and `gateway` pages can start with file-backed snapshots plus usage counts instead of full config editors

## Task 1: Lock Figma Tokens And Full Shell Navigation

**Files:**
- Create: `ui/src/lib/theme/product-shell.css`
- Modify: `ui/src/lib/app-shell/state.ts`
- Modify: `ui/src/routes/+layout.svelte`
- Test: `ui/tests/desktop_app_shell.spec.ts`

- [ ] **Step 1: Write the failing navigation test for the expanded shell**

```ts
import { expect, test } from "@playwright/test";

test("desktop shell exposes the full product navigation", async ({ page }) => {
    await page.goto("/workspace");

    await expect(page.getByRole("link", { name: /Workspace/i })).toBeVisible();
    await expect(page.getByRole("link", { name: /Agents/i })).toBeVisible();
    await expect(page.getByRole("link", { name: /Skills/i })).toBeVisible();
    await expect(page.getByRole("link", { name: /MCP/i })).toBeVisible();
    await expect(page.getByRole("link", { name: /Gateway/i })).toBeVisible();
});
```

- [ ] **Step 2: Run the test to verify it fails on missing routes**

Run: `pnpm --dir ui test:e2e --grep "desktop shell exposes the full product navigation"`
Expected: FAIL because the shell currently only renders `Setup`, `Workspace`, and `Skills`.

- [ ] **Step 3: Replace the hard-coded shell nav model with the Figma route family**

```ts
export const appShellNav: AppShellNavItem[] = [
    {
        href: "/workspace",
        label: "Workspace",
        caption: "Talk to the selected agent and monitor the current run.",
        shortcut: "Cmd-1"
    },
    {
        href: "/agents",
        label: "Agents",
        caption: "Browse agents and open per-agent configuration.",
        shortcut: "Cmd-2"
    },
    {
        href: "/skills",
        label: "Skills",
        caption: "Manage the global skills catalog.",
        shortcut: "Cmd-3"
    },
    {
        href: "/mcp",
        label: "MCP",
        caption: "Inspect shared MCP servers and connection health.",
        shortcut: "Cmd-4"
    },
    {
        href: "/gateway",
        label: "Gateway",
        caption: "Inspect global messaging gateways and routing posture.",
        shortcut: "Cmd-5"
    }
];
```

- [ ] **Step 4: Add shared Figma-derived light shell tokens**

```css
:root {
    color-scheme: light;
    --mc-bg: #fafafb;
    --mc-surface: #ffffff;
    --mc-raised: #f8f8fa;
    --mc-sunken: #f5f5f7;
    --mc-hover: #f2f3f6;
    --mc-text: #1a1d27;
    --mc-text-secondary: #4d5468;
    --mc-text-muted: #80879b;
    --mc-border: #e0e2e8;
    --mc-border-strong: #ced1da;
    --mc-primary: #6359f3;
    --mc-primary-600: #4e45d9;
    --mc-accent: #f54840;
    --mc-success: #22c566;
    --mc-warning: #fcc72b;
    --mc-danger: #ef3939;
    --mc-radius-input: 4px;
    --mc-radius-button: 8px;
    --mc-radius-card: 12px;
    --mc-radius-panel: 16px;
}
```

- [ ] **Step 5: Import the shared theme and restyle the top shell layout before touching page content**

```svelte
<script lang="ts">
    import "$lib/theme/product-shell.css";
    import { page } from "$app/stores";
    import { appShellNav, describeRoute, isActiveRoute } from "$lib/app-shell/state";
</script>
```

- [ ] **Step 6: Re-run shell checks**

Run: `pnpm --dir ui check && pnpm --dir ui test:e2e --grep "desktop shell exposes the full product navigation"`
Expected: PASS and the shell is now light-themed with the full route set.

- [ ] **Step 7: Commit**

```bash
git add ui/src/lib/theme/product-shell.css ui/src/lib/app-shell/state.ts ui/src/routes/+layout.svelte ui/tests/desktop_app_shell.spec.ts
git commit -m "feat: add figma shell navigation and light theme tokens"
```

## Task 2: Add Agent Profiles And Stable Session Bindings In `app-host`

**Files:**
- Create: `crates/app-host/src/agent_store.rs`
- Create: `crates/app-host/src/session_binding_store.rs`
- Modify: `crates/app-host/src/lib.rs`
- Modify: `crates/app-host/src/http/agent_api.rs`
- Test: `crates/app-host/tests/session_agent_binding.rs`
- Test: `crates/app-host/tests/agent_directory_contract.rs`

- [ ] **Step 1: Write the failing session binding test**

```rust
#[test]
fn session_agent_binding_rejects_drift() {
    let home = temp_home();

    bind_session_to_agent(&home, "session-a", "atlas").expect("bind initial agent");

    let same = bind_session_to_agent(&home, "session-a", "atlas").expect("rebind same agent");
    assert_eq!(same.agent_name, "atlas");

    let error = bind_session_to_agent(&home, "session-a", "scribe")
        .expect_err("drifting a session to a new agent should fail");
    assert!(error.to_string().contains("already bound"));
}
```

- [ ] **Step 2: Run the Rust test to verify the binding store does not exist yet**

Run: `cargo test -p matrixclaw-app-host session_agent_binding_rejects_drift -- --exact`
Expected: FAIL because `bind_session_to_agent` and the binding store file do not exist.

- [ ] **Step 3: Add a file-backed session binding store under runtime state**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionAgentBinding {
    pub session_id: String,
    pub agent_name: String,
}

pub fn bind_session_to_agent(
    home: impl AsRef<Path>,
    session_id: impl AsRef<str>,
    agent_name: impl AsRef<str>,
) -> io::Result<SessionAgentBinding> {
    let session_id = session_id.as_ref().trim().to_string();
    let agent_name = agent_name.as_ref().trim().to_string();
    let mut bindings = load_session_bindings(home.as_ref())?;

    if let Some(existing) = bindings.iter().find(|it| it.session_id == session_id) {
        if existing.agent_name != agent_name {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("session {session_id} already bound to {}", existing.agent_name),
            ));
        }
        return Ok(existing.clone());
    }

    let created = SessionAgentBinding { session_id, agent_name };
    bindings.push(created.clone());
    save_session_bindings(home.as_ref(), &bindings)?;
    Ok(created)
}
```

- [ ] **Step 4: Add a file-backed agent profile store that can summarize crown job, memory, and bindings**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentProfile {
    pub agent_name: String,
    pub title: String,
    pub crown_job: String,
    pub memory_summary: String,
    pub memory_signal_count: usize,
    pub pinned_memory_count: usize,
    pub enabled_skills: Vec<String>,
    pub enabled_mcp_servers: Vec<String>,
    pub enabled_gateways: Vec<String>,
}
```

- [ ] **Step 5: Teach the run API to accept the selected agent on first run and resolve the binding centrally**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRunRequest {
    pub prompt: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub agent_name: Option<String>,
}
```

And inside request normalization:

```rust
let requested_session_id = payload.session_id.clone().unwrap_or_else(new_session_id);
let requested_agent_name = payload
    .agent_name
    .clone()
    .unwrap_or_else(|| surface.current_agent_name());
let binding = bind_session_to_agent(surface.home(), &requested_session_id, &requested_agent_name)?;
```

- [ ] **Step 6: Add the failing agent directory test and then implement it from the profile store**

```rust
#[test]
fn agent_directory_contract_lists_profiles_with_binding_counts() {
    let home = seeded_home_with_agents();
    let agents = list_agent_profiles(&home).expect("load agent profiles");

    assert!(agents.iter().any(|agent| agent.agent_name == "atlas"));
    assert!(agents.iter().any(|agent| agent.enabled_skills.contains(&"web_search".to_string())));
}
```

Run: `cargo test -p matrixclaw-app-host agent_directory_contract_lists_profiles_with_binding_counts -- --exact`
Expected: FAIL first, then PASS once the profile store is implemented.

- [ ] **Step 7: Re-run both Rust tests**

Run: `cargo test -p matrixclaw-app-host session_agent_binding_rejects_drift -- --exact && cargo test -p matrixclaw-app-host agent_directory_contract_lists_profiles_with_binding_counts -- --exact`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add crates/app-host/src/agent_store.rs crates/app-host/src/session_binding_store.rs crates/app-host/src/http/agent_api.rs crates/app-host/src/lib.rs crates/app-host/tests/session_agent_binding.rs crates/app-host/tests/agent_directory_contract.rs
git commit -m "feat: add agent profiles and stable session bindings"
```

## Task 3: Add Global Catalog Endpoints For Skills, MCP, And Gateway

**Files:**
- Create: `crates/app-host/src/http/agents_api.rs`
- Create: `crates/app-host/src/http/mcp_api.rs`
- Create: `crates/app-host/src/http/gateway_api.rs`
- Modify: `crates/app-host/src/http/skills_api.rs`
- Modify: `crates/app-host/src/http/mod.rs`
- Test: `crates/app-host/tests/skills_catalog_contract.rs`
- Test: `crates/app-host/tests/mcp_catalog_contract.rs`
- Test: `crates/app-host/tests/gateway_catalog_contract.rs`

- [ ] **Step 1: Write the failing skills catalog test with enabled-by counts**

```rust
#[test]
fn skills_catalog_contract_reports_enabled_by_counts() {
    let home = seeded_home_with_agents_and_skills();
    let catalog = skills_catalog_for_home(&home).expect("load skills catalog");

    let research = catalog.iter().find(|item| item.name == "research").expect("research skill present");
    assert_eq!(research.enabled_by_agent_count, 2);
}
```

- [ ] **Step 2: Write the failing MCP and gateway catalog tests**

```rust
#[test]
fn mcp_catalog_contract_uses_file_backed_snapshot_or_defaults() {
    let home = temp_home();
    let catalog = mcp_catalog_for_home(&home).expect("load mcp catalog");
    assert!(!catalog.is_empty());
}

#[test]
fn gateway_catalog_contract_uses_file_backed_snapshot_or_defaults() {
    let home = temp_home();
    let catalog = gateway_catalog_for_home(&home).expect("load gateway catalog");
    assert!(!catalog.is_empty());
}
```

- [ ] **Step 3: Run the failing Rust tests**

Run: `cargo test -p matrixclaw-app-host skills_catalog_contract_reports_enabled_by_counts -- --exact`
Expected: FAIL.

Run: `cargo test -p matrixclaw-app-host mcp_catalog_contract_uses_file_backed_snapshot_or_defaults -- --exact`
Expected: FAIL.

Run: `cargo test -p matrixclaw-app-host gateway_catalog_contract_uses_file_backed_snapshot_or_defaults -- --exact`
Expected: FAIL.

- [ ] **Step 4: Add explicit global catalog DTOs and flat routes that match the handwritten router**

```rust
pub const AGENTS_DIRECTORY_ROUTE: &str = "/api/agents";
pub const AGENT_DETAIL_ROUTE: &str = "/api/agents/detail";
pub const SKILLS_CATALOG_ROUTE: &str = "/api/skills/catalog";
pub const MCP_CATALOG_ROUTE: &str = "/api/mcp";
pub const GATEWAY_CATALOG_ROUTE: &str = "/api/gateway";
```

- [ ] **Step 5: Extend the skills API with a global catalog view rather than removing the existing per-agent endpoint**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillCatalogRecord {
    pub name: String,
    pub source_root: PathBuf,
    pub installed_root: PathBuf,
    pub enabled_by_agent_count: usize,
    pub enabled_by_agents: Vec<String>,
}
```

- [ ] **Step 6: Add MCP and gateway snapshot loaders using file-backed JSON plus seeded defaults**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpCatalogRecord {
    pub name: String,
    pub health: String,
    pub enabled_by_agent_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatewayCatalogRecord {
    pub name: String,
    pub health: String,
    pub enabled_by_agent_count: usize,
}
```

- [ ] **Step 7: Register the new routes in `HttpRequest::handle` and return typed JSON from each**

```rust
if agents_api::is_agents_directory_route(&request.path) && request.method == HttpMethod::Get {
    return agents_api::agents_directory_response(self);
}

if agents_api::is_agent_detail_route(&request.path) && request.method == HttpMethod::Get {
    return agents_api::agent_detail_response(self, &request.path);
}

if mcp_api::is_mcp_catalog_route(&request.path) && request.method == HttpMethod::Get {
    return mcp_api::mcp_catalog_response(self);
}

if gateway_api::is_gateway_catalog_route(&request.path) && request.method == HttpMethod::Get {
    return gateway_api::gateway_catalog_response(self);
}
```

- [ ] **Step 8: Re-run the Rust contract tests**

Run: `cargo test -p matrixclaw-app-host skills_catalog_contract_reports_enabled_by_counts -- --exact && cargo test -p matrixclaw-app-host mcp_catalog_contract_uses_file_backed_snapshot_or_defaults -- --exact && cargo test -p matrixclaw-app-host gateway_catalog_contract_uses_file_backed_snapshot_or_defaults -- --exact`
Expected: PASS.

- [ ] **Step 9: Commit**

```bash
git add crates/app-host/src/http/agents_api.rs crates/app-host/src/http/mcp_api.rs crates/app-host/src/http/gateway_api.rs crates/app-host/src/http/skills_api.rs crates/app-host/src/http/mod.rs crates/app-host/tests/skills_catalog_contract.rs crates/app-host/tests/mcp_catalog_contract.rs crates/app-host/tests/gateway_catalog_contract.rs
git commit -m "feat: add global catalog APIs for shell v2"
```

## Task 4: Scaffold Typed Frontend Clients And New Routes

**Files:**
- Create: `ui/src/lib/agents/index.ts`
- Create: `ui/src/lib/agents/session.ts`
- Create: `ui/src/lib/catalogs/mcp.ts`
- Create: `ui/src/lib/catalogs/gateway.ts`
- Create: `ui/src/routes/agents/+page.svelte`
- Create: `ui/src/routes/agents/[agentId]/+page.svelte`
- Create: `ui/src/routes/mcp/+page.svelte`
- Create: `ui/src/routes/gateway/+page.svelte`
- Test: `ui/tests/agents_directory.spec.ts`
- Test: `ui/tests/agent_detail.spec.ts`
- Test: `ui/tests/global_catalog_pages.spec.ts`

- [ ] **Step 1: Write the failing agents directory Playwright test**

```ts
import { expect, test } from "@playwright/test";

test("agents route lists available agents and opens detail", async ({ page }) => {
    await page.route("**/api/agents", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify([
                {
                    agent_name: "atlas",
                    title: "Research Agent",
                    crown_job: "Research topics and synthesize findings.",
                    memory_signal_count: 14,
                    enabled_skills: ["web_search"],
                    enabled_mcp_servers: ["search-01"],
                    enabled_gateways: ["matrix"]
                }
            ])
        });
    });

    await page.goto("/agents");
    await expect(page.getByRole("heading", { name: "Agents" })).toBeVisible();
    await expect(page.getByText("Atlas")).toBeVisible();
});
```

- [ ] **Step 2: Write the failing agent detail and global catalog tests**

```ts
test("agent detail renders crown job, memory, and capability bindings", async ({ page }) => {
    await page.goto("/agents/atlas");
    await expect(page.getByRole("heading", { name: "Agent Detail" })).toBeVisible();
    await expect(page.getByText("Crown Job")).toBeVisible();
    await expect(page.getByText("Enabled Skills")).toBeVisible();
});

test("global catalog pages expose skills, mcp, and gateway views", async ({ page }) => {
    await page.goto("/skills");
    await expect(page.getByRole("heading", { name: "Skills" })).toBeVisible();
    await page.goto("/mcp");
    await expect(page.getByRole("heading", { name: "MCP" })).toBeVisible();
    await page.goto("/gateway");
    await expect(page.getByRole("heading", { name: "Gateway" })).toBeVisible();
});
```

- [ ] **Step 3: Run the Playwright tests to verify the routes do not exist yet**

Run: `pnpm --dir ui test:e2e --grep "agents route lists available agents and opens detail|agent detail renders crown job, memory, and capability bindings|global catalog pages expose skills, mcp, and gateway views"`
Expected: FAIL with route-not-found or missing heading assertions.

- [ ] **Step 4: Add typed client contracts first so page components stay small**

```ts
export type AgentSummary = {
    agent_name: string;
    title: string;
    crown_job: string;
    memory_signal_count: number;
    enabled_skills: string[];
    enabled_mcp_servers: string[];
    enabled_gateways: string[];
};

export async function fetchAgents(): Promise<AgentSummary[]> {
    return await fetchJson<AgentSummary[]>("/api/agents");
}
```

- [ ] **Step 5: Create minimal route shells that load data and render Figma-aligned headings before styling them heavily**

```svelte
<script lang="ts">
    import { onMount } from "svelte";
    import { fetchAgents, type AgentSummary } from "$lib/agents";

    let agents: AgentSummary[] = [];

    onMount(async () => {
        agents = await fetchAgents();
    });
</script>

<section>
    <h1>Agents</h1>
    {#each agents as agent}
        <a href={`/agents/${agent.agent_name}`}>{agent.title}</a>
    {/each}
</section>
```

- [ ] **Step 6: Re-run `svelte-check` and the new Playwright route tests**

Run: `pnpm --dir ui check && pnpm --dir ui test:e2e --grep "agents route lists available agents and opens detail|agent detail renders crown job, memory, and capability bindings|global catalog pages expose skills, mcp, and gateway views"`
Expected: PASS with minimal route content.

- [ ] **Step 7: Commit**

```bash
git add ui/src/lib/agents/index.ts ui/src/lib/agents/session.ts ui/src/lib/catalogs/mcp.ts ui/src/lib/catalogs/gateway.ts ui/src/routes/agents/+page.svelte ui/src/routes/agents/[agentId]/+page.svelte ui/src/routes/mcp/+page.svelte ui/src/routes/gateway/+page.svelte ui/tests/agents_directory.spec.ts ui/tests/agent_detail.spec.ts ui/tests/global_catalog_pages.spec.ts
git commit -m "feat: scaffold product shell v2 routes and typed clients"
```

## Task 5: Rebuild `/workspace` As The Figma Tri-Rail Shell

**Files:**
- Modify: `ui/src/routes/workspace/+page.svelte`
- Modify: `ui/src/lib/workspace/shell.ts`
- Modify: `ui/src/lib/agents/session.ts`
- Test: `ui/tests/workspace_tri_rail.spec.ts`
- Test: `ui/tests/workspace-streaming-transcript.spec.ts`
- Test: `ui/tests/ui-smoke.spec.ts`

- [ ] **Step 1: Write the failing tri-rail workspace test against the Figma headings and rails**

```ts
import { expect, test } from "@playwright/test";

test("workspace uses agent summary, conversation, and run state rails", async ({ page }) => {
    await page.goto("/workspace");

    await expect(page.getByText("Active Agent")).toBeVisible();
    await expect(page.getByText("Crown Job")).toBeVisible();
    await expect(page.getByRole("heading", { name: "Atlas" })).toBeVisible();
    await expect(page.getByText("Run State")).toBeVisible();
    await expect(page.getByPlaceholder("Message Atlas...")).toBeVisible();
});
```

- [ ] **Step 2: Run the workspace test to verify the old explorer-first page still fails the new contract**

Run: `pnpm --dir ui test:e2e --grep "workspace uses agent summary, conversation, and run state rails"`
Expected: FAIL because the left rail is still the workspace browser.

- [ ] **Step 3: Replace the old left explorer pane with an agent control summary that reads from the selected agent and session helpers**

```ts
export type SelectedAgentSession = {
    agentName: string;
    sessionId: string;
};

export const defaultSelectedAgentSession: SelectedAgentSession = {
    agentName: "atlas",
    sessionId: ""
};
```

- [ ] **Step 4: Keep file references, but move them into the composer attachment tray rather than a dedicated left browser column**

```svelte
<div class="composer-chip-row">
    {#each composerReferences as reference}
        <button type="button" class="reference-chip" on:click={() => promptDraft = `${promptDraft} ${reference}`.trim()}>
            {reference}
        </button>
    {/each}
</div>
```

- [ ] **Step 5: Send `agent_name` on first run so the backend can bind `session_id -> agent_name`**

```ts
const response = await fetch("/api/agent/run/stream", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
        prompt: promptDraft.trim(),
        session_id: sessionId || undefined,
        agent_name: selected.agentName
    })
});
```

- [ ] **Step 6: Replace the old queue diagnostic cards with the Figma run-state cards**

```ts
return {
    queueCards: [
        {
            title: "Queue",
            label: `${pendingCount}`,
            body: `${runningCount} running · ${queuedCount} queued`,
            tone: "neutral"
        }
    ],
    executionCards: [
        {
            title: "Execution Posture",
            label: execution.modeLabel,
            body: execution.sandboxPriority.join(" > "),
            tone: "neutral"
        },
        {
            title: "Warning",
            label: "runtime warning",
            body: execution.sandboxFailureMessage,
            tone: "warning"
        }
    ]
};
```

- [ ] **Step 7: Update the smoke tests to reflect the new shell instead of the old browser/stream/inspector headings**

```ts
await expect(page.getByText("Active Agent")).toBeVisible();
await expect(page.getByRole("heading", { name: "Atlas" })).toBeVisible();
await expect(page.getByText("Run State")).toBeVisible();
await expect(page.getByPlaceholder("Message Atlas..." )).toBeVisible();
```

- [ ] **Step 8: Re-run UI verification for the workspace shell**

Run: `pnpm --dir ui check && pnpm --dir ui test:e2e --grep "workspace uses agent summary, conversation, and run state rails|workspace transcript streams deltas without duplicating the final assistant message|browser smoke verifies live workspace and skills flows"`
Expected: PASS.

- [ ] **Step 9: Commit**

```bash
git add ui/src/routes/workspace/+page.svelte ui/src/lib/workspace/shell.ts ui/src/lib/agents/session.ts ui/tests/workspace_tri_rail.spec.ts ui/tests/workspace-streaming-transcript.spec.ts ui/tests/ui-smoke.spec.ts
git commit -m "feat: rebuild workspace as figma tri-rail shell"
```

## Task 6: Implement Agents Directory, Agent Detail, And Global Catalog Pages

**Files:**
- Modify: `ui/src/routes/agents/+page.svelte`
- Modify: `ui/src/routes/agents/[agentId]/+page.svelte`
- Modify: `ui/src/routes/skills/+page.svelte`
- Modify: `ui/src/routes/mcp/+page.svelte`
- Modify: `ui/src/routes/gateway/+page.svelte`
- Modify: `ui/src/lib/agents/index.ts`
- Modify: `ui/src/lib/catalogs/mcp.ts`
- Modify: `ui/src/lib/catalogs/gateway.ts`
- Test: `ui/tests/agents_directory.spec.ts`
- Test: `ui/tests/agent_detail.spec.ts`
- Test: `ui/tests/global_catalog_pages.spec.ts`

- [ ] **Step 1: Write the failing detail-page assertion for all major Figma sections**

```ts
await expect(page.getByText("Identity")).toBeVisible();
await expect(page.getByText("Crown Job")).toBeVisible();
await expect(page.getByText("Memory")).toBeVisible();
await expect(page.getByText("Enabled Skills")).toBeVisible();
await expect(page.getByText("Enabled MCP Servers")).toBeVisible();
await expect(page.getByText("Enabled Gateways")).toBeVisible();
```

- [ ] **Step 2: Run the Playwright agent-detail test to verify the page is still just a placeholder**

Run: `pnpm --dir ui test:e2e --grep "agent detail renders crown job, memory, and capability bindings"`
Expected: FAIL.

- [ ] **Step 3: Implement the agents directory using cards that mirror the design system component shapes**

```svelte
{#each agents as agent}
    <a class="agent-card" href={`/agents/${agent.agent_name}`}>
        <div>
            <strong>{agent.title}</strong>
            <p>{agent.crown_job}</p>
        </div>
        <div class="capability-pills">
            <span>Skills {agent.enabled_skills.length}</span>
            <span>MCP {agent.enabled_mcp_servers.length}</span>
            <span>GW {agent.enabled_gateways.length}</span>
        </div>
    </a>
{/each}
```

- [ ] **Step 4: Implement the agent detail page as the per-agent cockpit, not a generic admin list**

```svelte
<section class="agent-detail-grid">
    <article class="detail-panel">
        <p class="section-label">Crown Job</p>
        <h2>Crown Job</h2>
        <p>{detail.crown_job}</p>
    </article>

    <article class="detail-panel">
        <p class="section-label">Memory</p>
        <h2>Memory</h2>
        <p>{detail.memory_summary}</p>
    </article>
</section>
```

- [ ] **Step 5: Convert `/skills` into a global catalog page and remove direct per-agent toggling from that route**

```svelte
<span class="status-pill">Enabled by {selectedSkill.enabled_by_agent_count} agents</span>
<p>Manage per-agent enablement from Agent Detail.</p>
```

- [ ] **Step 6: Implement `/mcp` and `/gateway` as global catalog pages with usage counts and health badges**

```svelte
{#each items as item}
    <article class="catalog-row">
        <div>
            <strong>{item.name}</strong>
            <p>{item.health}</p>
        </div>
        <span>Enabled by {item.enabled_by_agent_count} agents</span>
    </article>
{/each}
```

- [ ] **Step 7: Re-run UI verification for all shell pages**

Run: `pnpm --dir ui check && pnpm --dir ui test:e2e --grep "agents route lists available agents and opens detail|agent detail renders crown job, memory, and capability bindings|global catalog pages expose skills, mcp, and gateway views"`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add ui/src/routes/agents/+page.svelte ui/src/routes/agents/[agentId]/+page.svelte ui/src/routes/skills/+page.svelte ui/src/routes/mcp/+page.svelte ui/src/routes/gateway/+page.svelte ui/src/lib/agents/index.ts ui/src/lib/catalogs/mcp.ts ui/src/lib/catalogs/gateway.ts ui/tests/agents_directory.spec.ts ui/tests/agent_detail.spec.ts ui/tests/global_catalog_pages.spec.ts
git commit -m "feat: implement agents and global catalog pages"
```

## Task 7: Full Verification, Fixture Refresh, And Docs Sync

**Files:**
- Modify: `crates/app-host/src/server.rs`
- Modify: `docs/superpowers/specs/2026-03-30-workspace-agent-configuration-design.md`
- Modify: `docs/superpowers/specs/2026-03-30-figma-design-handoff.md`
- Test: existing Rust + Playwright suites

- [ ] **Step 1: Refresh the seeded fixture data so smoke tests and screenshots resemble the Figma shell**

```rust
agent_name: "atlas".to_string(),
```

And seed richer profile/catalog defaults for:
- `atlas`
- at least one alternate agent such as `scribe`
- one MCP entry
- one gateway entry

- [ ] **Step 2: Run targeted Rust verification for the new backend surface**

Run: `cargo test -p matrixclaw-app-host session_agent_binding_rejects_drift -- --exact && cargo test -p matrixclaw-app-host agent_directory_contract_lists_profiles_with_binding_counts -- --exact && cargo test -p matrixclaw-app-host skills_catalog_contract_reports_enabled_by_counts -- --exact && cargo test -p matrixclaw-app-host mcp_catalog_contract_uses_file_backed_snapshot_or_defaults -- --exact && cargo test -p matrixclaw-app-host gateway_catalog_contract_uses_file_backed_snapshot_or_defaults -- --exact`
Expected: PASS.

- [ ] **Step 3: Run targeted Playwright verification for the shell redesign**

Run: `pnpm --dir ui test:e2e --grep "desktop shell exposes the full product navigation|workspace uses agent summary, conversation, and run state rails|agents route lists available agents and opens detail|agent detail renders crown job, memory, and capability bindings|global catalog pages expose skills, mcp, and gateway views|browser smoke verifies live workspace and skills flows"`
Expected: PASS.

- [ ] **Step 4: Run type-check and broader package validation**

Run: `pnpm --dir ui check && cargo test -p matrixclaw-app-host`
Expected: PASS.

- [ ] **Step 5: Sync the spec docs so the repo source of truth references the light Figma design system instead of the old dark appendix**

```md
Implementation note: the Figma light design system file is the visual source of truth for shell v2. The route and ownership model in this spec remains valid, but the earlier dark token section is superseded by the newer Figma design file.
```

- [ ] **Step 6: Commit**

```bash
git add crates/app-host/src/server.rs docs/superpowers/specs/2026-03-30-workspace-agent-configuration-design.md docs/superpowers/specs/2026-03-30-figma-design-handoff.md
git commit -m "docs: sync shell v2 plan with figma source of truth"
```

## Self-Review

### Spec coverage
- Chat-first `/workspace`: covered by Task 5.
- Explicit agent identity and per-agent config: covered by Tasks 2, 4, and 6.
- Global `skills`, `mcp`, `gateway` separation: covered by Tasks 3 and 6.
- Runtime-visible but non-admin right rail: covered by Task 5.
- Agent/session architecture fit: covered by Task 2.
- Figma design-system alignment: covered by Tasks 1 and 7.

### Placeholder scan
- No `TODO`, `TBD`, or “implement later” markers remain.
- Each coding task includes concrete file paths, commands, and code snippets.
- Each test-first task includes an explicit failing test before implementation.

### Type consistency
- Backend uses `agent_name` consistently at the shell/API layer.
- `session_id -> agent_name` binding lives in `app-host`, not `session-runtime`.
- Frontend route family consistently uses `/agents`, `/agents/[agentId]`, `/skills`, `/mcp`, `/gateway`.

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-03-30-product-shell-v2-from-figma.md`. Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
