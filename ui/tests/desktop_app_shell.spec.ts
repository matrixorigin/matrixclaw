import { expect, test } from "@playwright/test";

const workspaceFiles = [
    {
        relative_path: "src/main.rs",
        kind: "File",
        reference_token: "[[workspace:src/main.rs]]"
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

test("desktop shell exposes the full product navigation", async ({
    page
}) => {
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

    await page.route("**/api/queue/state*", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(queueState)
        });
    });

    await page.route("**/api/execution/visibility", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(executionVisibility)
        });
    });

    await page.goto("/workspace");

    await expect(page.getByRole("heading", { name: "MatrixClaw" }).first()).toBeVisible();
    await expect(page.getByRole("link", { name: /Workspace Cmd-1/i })).toHaveAttribute("href", "/workspace");
    await expect(page.getByRole("link", { name: /Agents Cmd-2/i })).toHaveAttribute("href", "/agents");
    await expect(page.getByRole("link", { name: /Skills Cmd-3/i })).toHaveAttribute("href", "/skills");
    await expect(page.getByRole("link", { name: /MCP Cmd-4/i })).toHaveAttribute("href", "/mcp");
    await expect(page.getByRole("link", { name: /Gateway Cmd-5/i })).toHaveAttribute("href", "/gateway");
    await expect(page.getByText("Active Agent", { exact: true })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Conversation" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Run State" })).toBeVisible();
});
