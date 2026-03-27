import { expect, test } from "@playwright/test";

type WorkspaceFilesPayload = Array<{
    relative_path: string;
    kind: "File" | "Directory";
    reference_token: string;
}>;

type QueueStatePayload = {
    steering: {
        kind: "steering";
        submit_route: string;
        delivery_timing: "next-turn" | "next-run" | "queued";
        summary: string;
    };
    follow_up: {
        kind: "follow-up";
        submit_route: string;
        delivery_timing: "next-turn" | "next-run" | "queued";
        summary: string;
    };
};

type ExecutionVisibilityPayload = {
    modeLabel: string;
    visibleBackends: string[];
    sandboxPriority: string[];
    sandboxFailureMessage: string;
    fallbackPolicy: string;
};

const workspaceFiles: WorkspaceFilesPayload = [
    {
        relative_path: "src/main.rs",
        kind: "File",
        reference_token: "[[workspace:src/main.rs]]"
    },
    {
        relative_path: "src/lib.rs",
        kind: "File",
        reference_token: "[[workspace:src/lib.rs]]"
    },
    {
        relative_path: "docs",
        kind: "Directory",
        reference_token: "[[workspace:docs]]"
    }
];

const queueState: QueueStatePayload = {
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

const executionVisibility: ExecutionVisibilityPayload = {
    modeLabel: "local",
    visibleBackends: ["local", "docker", "boxlite"],
    sandboxPriority: ["docker", "boxlite"],
    sandboxFailureMessage: "Sandbox-only operations remain explicit in the workspace shell.",
    fallbackPolicy: "prefer-sandbox"
};

test("workspace transcript streams deltas without duplicating the final assistant message", async ({
    page
}) => {
    let firstSessionId = "";
    let queueStateSeenSessionId = false;
    let streamRequestCount = 0;

    await page.route("**/api/workspace/files", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(workspaceFiles)
        });
    });

    await page.route("**/api/queue/state*", async (route) => {
        queueStateSeenSessionId =
            !!new URL(route.request().url()).searchParams.get("session_id") || queueStateSeenSessionId;
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
        const body = route.request().postDataJSON() as { prompt?: string; session_id?: string };
        if (!firstSessionId) {
            expect(body.session_id).toBeTruthy();
            firstSessionId = body.session_id ?? "";
        } else {
            expect(body.session_id).toBe(firstSessionId);
        }

        streamRequestCount += 1;
        const finalMessage = `Final assistant answer ${streamRequestCount}`;
        const streamedMessage = `Drafting chunk ${streamRequestCount} of 2`;
        const frames = [
            {
                type: "event",
                event: { sequence: 0, kind: "message_started" }
            },
            {
                type: "event",
                event: { sequence: 1, kind: "message_delta", content: streamedMessage }
            },
            {
                type: "event",
                event: { sequence: 2, kind: "message_completed", content: finalMessage }
            },
            {
                type: "complete",
                session_id: firstSessionId,
                model: "mock-stream",
                streamed_message: streamedMessage,
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
    await expect(page.getByText("Visible backends")).toBeVisible();
    await expect(page.getByText("Runtime contract")).toBeVisible();
    expect(queueStateSeenSessionId).toBeTruthy();

    await page.getByLabel("Composer").fill("Stream the answer in two deltas.");
    await page.getByRole("button", { name: "Send" }).click();

    await expect(
        page.locator('article[data-role="assistant"]').filter({
            hasText: /^Final assistant answer 1$/
        })
    ).toHaveCount(1);

    await page.getByLabel("Composer").fill("Run the next turn on the same session.");
    await page.getByRole("button", { name: "Send" }).click();
    await expect(
        page.locator('article[data-role="assistant"]').filter({
            hasText: /^Final assistant answer 2$/
        }).first()
    ).toBeVisible();

    await expect(page.getByText("Visible backends")).toBeVisible();
    await expect(page.getByText("Runtime contract")).toBeVisible();
});
