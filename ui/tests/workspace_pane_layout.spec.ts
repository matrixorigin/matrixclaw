import { expect, test } from "@playwright/test";

const workspaceFiles = [
    {
        relative_path: "src/main.rs",
        kind: "File",
        reference_token: "[[workspace:src/main.rs]]"
    },
    {
        relative_path: "README.md",
        kind: "File",
        reference_token: "[[workspace:README.md]]"
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
    modeLabel: "sandboxed",
    visibleBackends: ["docker", "boxlite", "local"],
    sandboxPriority: ["docker", "boxlite"],
    sandboxFailureMessage: "Sandbox-only operations remain explicit in the workspace shell.",
    fallbackPolicy: "prefer-sandbox"
};

test("workspace tri-rail layout keeps the agent, conversation, and run state visible", async ({
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

    const dock = page.getByTestId("workspace-control-dock");
    const transcript = page.getByTestId("workspace-conversation-column");
    const runState = page.getByTestId("workspace-run-state");

    await expect(dock).toBeVisible();
    await expect(transcript).toBeVisible();
    await expect(runState).toBeVisible();
    await expect(dock.getByRole("link", { name: "MCP" })).toBeVisible();
    await expect(dock.getByText("14 signals", { exact: false })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Conversation" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Run State" })).toBeVisible();
});
