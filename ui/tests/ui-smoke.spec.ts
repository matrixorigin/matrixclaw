import path from "node:path";

import { expect, test } from "@playwright/test";

const artifactDir = path.resolve(process.cwd(), "..", "output", "playwright");

const healthSnapshot = {
    mode: "setup",
    baseUrl: "http://127.0.0.1:38495",
    configReady: true
};

const workspaceFiles = [
    {
        relative_path: "src/main.rs",
        kind: "File",
        reference_token: "[[workspace:src/main.rs]]"
    },
    {
        relative_path: "src/lib.rs",
        kind: "File",
        reference_token: "[[workspace:src/lib.rs]]"
    },
    {
        relative_path: "docs",
        kind: "Directory",
        reference_token: "[[workspace:docs]]"
    }
];

const activeAgent = {
    agent_name: "atlas",
    title: "Atlas",
    crown_job: "Research topics and synthesize findings.",
    memory_summary: "Keeps long-running workspace context.",
    memory_signal_count: 14,
    pinned_memory_count: 3,
    enabled_skills: ["web_search"],
    enabled_mcp_servers: ["search-01"],
    enabled_gateways: ["matrix"],
    binding_count: 2
};

const queueState = {
    steering: {
        kind: "steering",
        submit_route: "/api/queue/steering",
        delivery_timing: "next-turn",
        summary: "Steering instructions are queued for the next turn."
    },
    follow_up: {
        kind: "follow-up",
        submit_route: "/api/queue/follow-up",
        delivery_timing: "next-run",
        summary: "Follow-up instructions wait until the next run completes."
    }
};

const executionVisibility = {
    modeLabel: "local",
    visibleBackends: ["local", "docker", "boxlite"],
    sandboxPriority: ["docker", "boxlite"],
    sandboxFailureMessage: "Sandbox-only operations remain explicit in the workspace shell.",
    fallbackPolicy: "prefer-sandbox"
};

const skillsCatalog = [
    {
        name: "research",
        source_root: "/imports/research",
        installed_root: "/runtime/skills/research",
        enabled_by_agent_count: 2,
        enabled_by_agents: ["atlas", "scribe"]
    },
    {
        name: "lint-bridge",
        source_root: "/imports/lint-bridge",
        installed_root: "/runtime/skills/lint-bridge",
        enabled_by_agent_count: 0,
        enabled_by_agents: []
    }
];

test("browser smoke verifies live workspace and skills flows", async ({ page }) => {
    let queueSessionId = "";

    await page.route("**/healthz", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(healthSnapshot)
        });
    });

    await page.route("**/api/setup/config", async (route) => {
        if (route.request().method() === "POST") {
            await route.fulfill({
                contentType: "application/json",
                body: JSON.stringify({
                    accepted: true,
                    configWritten: true,
                    next: "/workspace",
                    error: null
                })
            });
            return;
        }

        await route.fulfill({
            status: 405,
            contentType: "application/json",
            body: JSON.stringify({
                error: "setup submissions require POST"
            })
        });
    });

    await page.route("**/api/workspace/files", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(workspaceFiles)
        });
    });

    await page.route("**/api/agents/detail**", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(activeAgent)
        });
    });

    await page.route("**/api/workspace/reference", async (route) => {
        const body = route.request().postDataJSON() as { relative_path?: string };
        const referenceToken =
            body.relative_path === "src/lib.rs"
                ? "[[workspace:src/lib.rs]]"
                : "[[workspace:src/main.rs]]";

        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify({
                relative_path: body.relative_path ?? "src/main.rs",
                reference_token: referenceToken
            })
        });
    });

    await page.route("**/api/queue/state*", async (route) => {
        const sessionId = new URL(route.request().url()).searchParams.get("session_id");
        if (sessionId) {
            queueSessionId = sessionId;
        }
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(queueState)
        });
    });

    await page.route("**/api/queue/steering", async (route) => {
        const payload = route.request().postDataJSON() as { session_id?: string };
        expect(payload.session_id).toBeTruthy();
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify({
                accepted: true,
                session_id: payload.session_id ?? queueSessionId ?? "workspace-session",
                state: queueState.steering
            })
        });
    });

    await page.route("**/api/queue/follow-up", async (route) => {
        const payload = route.request().postDataJSON() as { session_id?: string };
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify({
                accepted: true,
                session_id: payload.session_id ?? queueSessionId ?? "workspace-session",
                state: queueState.follow_up
            })
        });
    });

    await page.route("**/api/execution/visibility", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(executionVisibility)
        });
    });

    await page.route("**/api/skills/catalog", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(skillsCatalog)
        });
    });

    await page.goto("/setup");
    await expect(page.getByRole("heading", { name: "Desktop first-launch setup" })).toBeVisible();
    await page.screenshot({ path: path.join(artifactDir, "setup.png"), fullPage: true });

    await page.goto("/workspace");
    await expect(page.getByText("Active Agent", { exact: true })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Atlas" }).first()).toBeVisible();
    await expect(page.getByRole("heading", { name: "Run State" })).toBeVisible();
    await expect(page.getByPlaceholder("Message Atlas...")).toBeVisible();
    expect(queueSessionId).toBeTruthy();
    await page.getByRole("button", { name: "Reference" }).first().click({ force: true });
    await expect(page.getByText("[[workspace:src/main.rs]]").first()).toBeVisible();

    const steeringMessage = "Playwright steering smoke message";
    const steeringArea = page.getByPlaceholder("Queue the next-turn steering instruction.");
    await steeringArea.fill(steeringMessage);
    await page.getByRole("button", { name: "Queue steering" }).click();
    await expect(page.getByText(steeringMessage)).toBeVisible();
    await page.screenshot({ path: path.join(artifactDir, "workspace.png"), fullPage: true });

    await page.goto("/skills");
    await expect(page.getByRole("main").getByRole("heading", { name: "Skills", level: 1 })).toBeVisible();
    await expect(page.getByRole("button", { name: /research/i })).toBeVisible();
    await expect(page.getByRole("button", { name: /lint-bridge/i })).toBeVisible();
    await page.getByRole("button", { name: /lint-bridge/i }).click();
    await expect(page.getByText("No agents are currently using this skill.")).toBeVisible();
    await expect(page.getByText("Agent Detail", { exact: true })).toBeVisible();
    await page.screenshot({ path: path.join(artifactDir, "skills.png"), fullPage: true });
});
