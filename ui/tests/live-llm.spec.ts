import path from "node:path";

import { expect, test } from "@playwright/test";

const artifactDir = path.resolve(process.cwd(), "..", "output", "playwright");
const enabled = process.env.MATRIXCLAW_LIVE_E2E === "1";
const expected = process.env.MATRIXCLAW_LIVE_SENTINEL ?? "MATRIXCLAW_UI_E2E_OK";

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

test.skip(!enabled, "live LLM smoke only runs when MATRIXCLAW_LIVE_E2E=1");

test("workspace composer reaches live LLM provider", async ({ page }) => {
    let sessionId = "";

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

    await page.route("**/api/agent/run/stream", async (route) => {
        const body = route.request().postDataJSON() as {
            prompt?: string;
            session_id?: string;
            agent_name?: string;
        };
        expect(body.agent_name).toBe("atlas");
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
    await expect(page.getByText("Conversation")).toBeVisible();

    await page.getByLabel("Composer").fill(`Reply with exactly ${expected} and nothing else.`);
    await page.getByRole("button", { name: "Send" }).click();

    await expect(page.locator("article").filter({ hasText: expected }).first()).toBeVisible({
        timeout: 30_000
    });

    await page.screenshot({
        path: path.join(artifactDir, "live-agent.png"),
        fullPage: true
    });
});
