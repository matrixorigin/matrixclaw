import { expect, test } from "@playwright/test";

type WorkspaceFilesPayload = Array<{
    relative_path: string;
    kind: "File" | "Directory";
    reference_token: string;
}>;

type QueueStatePayload = {
    steering: {
        kind: "manual" | "automatic";
        submit_route: string;
        delivery_timing: "next-turn" | "next-run" | "queued";
        summary: string;
    };
    follow_up: {
        kind: "manual" | "automatic";
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
        kind: "manual",
        submit_route: "/api/queue/steering",
        delivery_timing: "next-turn",
        summary: "Steering instructions are queued for the next turn."
    },
    follow_up: {
        kind: "manual",
        submit_route: "/api/queue/follow-up",
        delivery_timing: "next-run",
        summary: "Follow-up instructions wait until the next run completes."
    }
};

const executionVisibility: ExecutionVisibilityPayload = {
    modeLabel: "loopback",
    visibleBackends: ["loopback", "sandbox"],
    sandboxPriority: ["sandbox", "loopback"],
    sandboxFailureMessage: "Sandbox-only operations remain explicit in the workspace shell.",
    fallbackPolicy: "prefer-sandbox"
};

test("workspace transcript streams deltas without duplicating the final assistant message", async ({
    page
}) => {
    await page.route("**/api/workspace/files", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(workspaceFiles)
        });
    });

    await page.route("**/api/queue/state", async (route) => {
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

    await page.route("**/api/agent/run", async (route) => {
        await page.waitForTimeout(250);
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify({
                model: "mock-stream",
                streamed_message: "Drafting chunk 1 of 2",
                final_message: "Final assistant answer"
            })
        });
    });

    await page.goto("/workspace");

    await expect(page.getByRole("heading", { name: "Files and references" })).toBeVisible();
    await expect(page.getByText("Visible backends")).toBeVisible();
    await expect(page.getByText("Runtime contract")).toBeVisible();

    await page.getByLabel("Composer").fill("Stream the answer in two deltas.");
    await page.getByRole("button", { name: "Send" }).click();

    await expect(
        page.locator('article[data-role="assistant"]').filter({
            hasText: /^Drafting chunk 1 of 2$/
        })
    ).toBeVisible({ timeout: 500 });

    await expect(
        page.locator('article[data-role="assistant"]').filter({
            hasText: /^Final assistant answer$/
        })
    ).toHaveCount(1);

    await expect(page.getByText("Visible backends")).toBeVisible();
    await expect(page.getByText("Runtime contract")).toBeVisible();
});
