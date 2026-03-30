import { expect, test } from "@playwright/test";

const skillsCatalog = [
    {
        name: "research",
        enabled_by_agent_count: 2,
        enabled_by_agents: ["atlas", "beta"],
        source_root: "/imports/research",
        installed_root: "/runtime/skills/research"
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
    await page.route("**/api/skills?agent=default", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify({
                installed: [],
                enabled: []
            })
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
    await expect(
        page.getByRole("main").getByRole("heading", { name: "Global inventory", level: 2 })
    ).toBeVisible();

    await page.goto("/mcp");
    await expect(page.getByRole("main").getByRole("heading", { name: "MCP", level: 1 })).toBeVisible();
    await expect(page.getByText("search-01")).toBeVisible();

    await page.goto("/gateway");
    await expect(
        page.getByRole("main").getByRole("heading", { name: "Gateway", level: 1 })
    ).toBeVisible();
    await expect(page.getByRole("main").getByText("matrix", { exact: true })).toBeVisible();
});
