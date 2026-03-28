import { expect, test } from "@playwright/test";

test("setup onboarding persists draft state and resumes on reload", async ({ page }) => {
    await page.route("**/healthz", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify({
                mode: "setup",
                baseUrl: "http://127.0.0.1:38495/setup",
                configReady: false
            })
        });
    });

    await page.route("**/api/setup/config", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify({
                accepted: true,
                configWritten: true,
                next: "/workspace",
                error: null
            })
        });
    });

    await page.goto("/setup");
    await expect(page.getByRole("heading", { name: "Desktop first-launch setup" })).toBeVisible();

    await page.getByPlaceholder("openai-compatible").fill("openrouter");
    await page.getByPlaceholder("gpt-5.4").fill("moonshotai/kimi-k2.5");
    await page.getByRole("button", { name: "Continue" }).click();

    await expect(page.getByRole("heading", { name: "Workspace binding" })).toBeVisible();
    await page.reload();
    await expect(page.getByRole("heading", { name: "Workspace binding" })).toBeVisible();
    await expect(page.getByText("openrouter")).toBeVisible();
    await expect(page.getByText("moonshotai/kimi-k2.5")).toBeVisible();
});
