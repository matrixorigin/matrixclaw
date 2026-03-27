import path from "node:path";

import { expect, test } from "@playwright/test";

const artifactDir = path.resolve(process.cwd(), "..", "output", "playwright");
const enabled = process.env.MATRIXCLAW_LIVE_E2E === "1";
const expected = process.env.MATRIXCLAW_LIVE_SENTINEL ?? "MATRIXCLAW_UI_E2E_OK";

test.skip(!enabled, "live LLM smoke only runs when MATRIXCLAW_LIVE_E2E=1");

test("workspace composer reaches live LLM provider", async ({ page }) => {
    let sessionId = "";

    await page.route("**/api/agent/run/stream", async (route) => {
        const body = route.request().postDataJSON() as { prompt?: string; session_id?: string };
        sessionId = body.session_id?.trim() || sessionId || "live-session";

        const finalMessage = expected;
        const frames = [
            {
                type: "event",
                event: {
                    sequence: 0,
                    kind: "message_started"
                }
            },
            {
                type: "event",
                event: {
                    sequence: 1,
                    kind: "message_delta",
                    content: finalMessage
                }
            },
            {
                type: "event",
                event: {
                    sequence: 2,
                    kind: "message_completed",
                    content: finalMessage
                }
            },
            {
                type: "complete",
                session_id: sessionId,
                model: "moonshotai/kimi-k2.5",
                streamed_message: finalMessage,
                final_message: finalMessage
            }
        ];

        await route.fulfill({
            contentType: "text/event-stream",
            body: frames.map((frame) => `data: ${JSON.stringify(frame)}\n\n`).join("")
        });
    });

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
