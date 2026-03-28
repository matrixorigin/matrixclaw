import { expect, test } from "@playwright/test";

const workspaceFiles = [
    {
        relative_path: "src/main.rs",
        kind: "File",
        reference_token: "[[workspace:src/main.rs]]"
    },
    {
        relative_path: "README.md",
        kind: "File",
        reference_token: "[[workspace:README.md]]"
    }
];

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
    modeLabel: "sandboxed",
    visibleBackends: ["docker", "boxlite", "local"],
    sandboxPriority: ["docker", "boxlite"],
    sandboxFailureMessage: "Sandbox-only operations remain explicit in the workspace shell.",
    fallbackPolicy: "prefer-sandbox"
};

test("workspace uses desktop panes for browser, stream, and inspector", async ({ page }) => {
    await page.route("**/api/workspace/files", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(workspaceFiles)
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

    await page.goto("/workspace");

    await expect(page.getByRole("heading", { name: "Workspace browser" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Assistant stream" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Queue and execution detail" })).toBeVisible();
    await expect(page.getByText("Working set")).toBeVisible();
    await expect(page.getByText("Reference tray")).toBeVisible();
    await expect(page.getByText("Steering queue")).toBeVisible();
    await expect(page.getByText("Sandbox policy")).toBeVisible();
});
