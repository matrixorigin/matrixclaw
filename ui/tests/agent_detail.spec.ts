import { expect, test } from "@playwright/test";

const agentDetail = {
    agent_name: "atlas",
    title: "Research Agent",
    crown_job: "Research topics and synthesize findings.",
    memory_summary: "Keeps long-running research context.",
    memory_signal_count: 14,
    pinned_memory_count: 3,
    enabled_skills: ["web_search", "summarize"],
    enabled_mcp_servers: ["search-01"],
    enabled_gateways: ["matrix"],
    binding_count: 2
};

test("agent detail renders crown job, memory, and capability bindings", async ({ page }) => {
    await page.route("**/api/agents/detail**", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(agentDetail)
        });
    });

    await page.goto("/agents/atlas");

    await expect(
        page.getByRole("main").getByRole("heading", { name: "Agent Detail", level: 1 })
    ).toBeVisible();
    await expect(page.getByText("Identity", { exact: true })).toBeVisible();
    await expect(page.getByRole("main").getByRole("heading", { name: "Crown Job", level: 2 })).toBeVisible();
    await expect(page.getByText(agentDetail.crown_job)).toBeVisible();
    await expect(page.getByRole("main").getByRole("heading", { name: "Memory", level: 2 })).toBeVisible();
    await expect(page.getByText(agentDetail.memory_summary)).toBeVisible();
    await expect(
        page.getByRole("main").getByRole("heading", { name: "Enabled Skills", level: 2 })
    ).toBeVisible();
    await expect(page.getByText("web_search")).toBeVisible();
    await expect(
        page.getByRole("main").getByRole("heading", { name: "Enabled MCP Servers", level: 2 })
    ).toBeVisible();
    await expect(page.getByText("search-01")).toBeVisible();
    await expect(
        page.getByRole("main").getByRole("heading", { name: "Enabled Gateways", level: 2 })
    ).toBeVisible();
    await expect(page.getByRole("main").getByText("matrix", { exact: true })).toBeVisible();
});
