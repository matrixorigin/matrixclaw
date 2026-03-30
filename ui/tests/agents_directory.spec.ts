import { expect, test } from "@playwright/test";

const agents = [
    {
        agent_name: "atlas",
        title: "Research Agent",
        crown_job: "Research topics and synthesize findings.",
        memory_summary: "Keeps long-running research context.",
        memory_signal_count: 14,
        pinned_memory_count: 3,
        enabled_skills: ["web_search"],
        enabled_mcp_servers: ["search-01"],
        enabled_gateways: ["matrix"],
        binding_count: 2
    },
    {
        agent_name: "beta",
        title: "Draft Agent",
        crown_job: "Shape first-pass responses.",
        memory_summary: "Prefers short-lived context.",
        memory_signal_count: 6,
        pinned_memory_count: 1,
        enabled_skills: ["summarize"],
        enabled_mcp_servers: [],
        enabled_gateways: [],
        binding_count: 0
    }
];

test("agents route lists available agents and opens detail", async ({ page }) => {
    await page.route("**/api/agents", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(agents)
        });
    });

    await page.route("**/api/agents/detail**", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(agents[0])
        });
    });

    await page.goto("/agents");

    await expect(
        page.getByRole("main").getByRole("heading", { name: "Agents", level: 1 })
    ).toBeVisible();
    await expect(page.getByText("Research Agent")).toBeVisible();
    await expect(page.getByRole("link", { name: /Research Agent/ })).toHaveAttribute(
        "href",
        "/agents/atlas"
    );

    await page.getByRole("link", { name: /Research Agent/ }).click();

    await expect(
        page.getByRole("main").getByRole("heading", { name: "Agent Detail", level: 1 })
    ).toBeVisible();
    await expect(page.getByRole("main").getByRole("heading", { name: "Crown Job", level: 2 })).toBeVisible();
    await expect(
        page.getByRole("main").getByRole("heading", { name: "Enabled Skills", level: 2 })
    ).toBeVisible();
});
