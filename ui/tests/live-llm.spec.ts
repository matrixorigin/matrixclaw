import path from "node:path";

import { expect, test } from "@playwright/test";

const artifactDir = path.resolve(process.cwd(), "..", "output", "playwright");
const enabled = process.env.MATRIXCLAW_LIVE_E2E === "1";
const expected = process.env.MATRIXCLAW_LIVE_SENTINEL ?? "MATRIXCLAW_UI_E2E_OK";

test.skip(!enabled, "live LLM smoke only runs when MATRIXCLAW_LIVE_E2E=1");

test("workspace composer reaches live LLM provider", async ({ page }) => {
    await page.goto("/workspace");
    await expect(page.getByRole("heading", { name: "Files and references" })).toBeVisible();

    await page
        .getByLabel("Composer")
        .fill(`Reply with exactly ${expected} and nothing else.`);
    await page.getByRole("button", { name: "Send" }).click();

    await expect(
        page.locator("article").filter({ hasText: expected }).first()
    ).toBeVisible({ timeout: 30_000 });

    await page.screenshot({
        path: path.join(artifactDir, "live-agent.png"),
        fullPage: true
    });
});
