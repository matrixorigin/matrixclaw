import { expect, test } from "@playwright/test";

const skillsCatalog = [
    {
        name: "research",
        enabled_by_agent_count: 2,
        enabled_by_agents: ["atlas", "beta"],
        source_root: "/imports/research",
        installed_root: "/runtime/skills/research"
    },
    {
        name: "lint-bridge",
        enabled_by_agent_count: 0,
        enabled_by_agents: [],
        source_root: "/imports/lint-bridge",
        installed_root: "/runtime/skills/lint-bridge"
    }
];

const mcpCatalog = [
    {
        name: "search-01",
        health: "healthy",
        enabled_by_agent_count: 1
    }
];

const gatewayCatalog = [
    {
        name: "matrix",
        health: "healthy",
        enabled_by_agent_count: 1
    }
];

test("global catalog pages expose skills, mcp, and gateway views", async ({ page }) => {
    await page.route("**/api/skills/catalog", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(skillsCatalog)
        });
    });

    await page.route("**/api/mcp", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(mcpCatalog)
        });
    });

    await page.route("**/api/gateway", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(gatewayCatalog)
        });
    });

    await page.goto("/skills");
    await expect(page.getByRole("main").getByRole("heading", { name: "Skills", level: 1 })).toBeVisible();
    await expect(page.getByRole("button", { name: /research/i })).toBeVisible();
    await expect(page.getByText("Managed globally, enabled per agent.")).toBeVisible();
    await expect(page.getByRole("button", { name: /lint-bridge/i })).toBeVisible();
    await page.getByRole("button", { name: /lint-bridge/i }).click();
    await expect(page.getByText("No agents are currently using this skill.")).toBeVisible();

    await page.goto("/mcp");
    await expect(page.getByRole("main").getByRole("heading", { name: "MCP", level: 1 })).toBeVisible();
    await expect(page.getByText("search-01")).toBeVisible();
    await expect(page.getByText("Managed centrally, enabled per agent.").first()).toBeVisible();

    await page.goto("/gateway");
    await expect(
        page.getByRole("main").getByRole("heading", { name: "Gateway", level: 1 })
    ).toBeVisible();
    await expect(page.getByRole("main").getByText("matrix", { exact: true })).toBeVisible();
    await expect(page.getByText("Managed centrally, enabled per agent.").first()).toBeVisible();
});
