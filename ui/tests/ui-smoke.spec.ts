import path from "node:path";

import { expect, test } from "@playwright/test";

const artifactDir = path.resolve(process.cwd(), "..", "output", "playwright");

const healthSnapshot = {
    mode: "setup",
    baseUrl: "http://127.0.0.1:38495",
    configReady: true
};

const workspaceFiles = [
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

const queueState = {
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

const executionVisibility = {
    modeLabel: "loopback",
    visibleBackends: ["loopback", "sandbox"],
    sandboxPriority: ["sandbox", "loopback"],
    sandboxFailureMessage: "Sandbox-only operations remain explicit in the workspace shell.",
    fallbackPolicy: "prefer-sandbox"
};

const skillsInventory = {
    installed: [
        {
            name: "research",
            source_root: "/imports/research",
            installed_root: "/runtime/skills/research",
            manifest_path: "/runtime/skills/research/matrixclaw.skill.json",
            provenance_path: "/runtime/skills/research/provenance.json"
        },
        {
            name: "lint-bridge",
            source_root: "/imports/lint-bridge",
            installed_root: "/runtime/skills/lint-bridge",
            manifest_path: "/runtime/skills/lint-bridge/matrixclaw.skill.json",
            provenance_path: "/runtime/skills/lint-bridge/provenance.json"
        }
    ],
    enabled: [
        {
            agent_name: "default",
            enabled: []
        }
    ]
};

test("browser smoke verifies live workspace and skills flows", async ({ page }) => {
    await page.route("**/healthz", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(healthSnapshot)
        });
    });

    await page.route("**/api/setup/config", async (route) => {
        if (route.request().method() === "POST") {
            await route.fulfill({
                contentType: "application/json",
                body: JSON.stringify({
                    accepted: true,
                    configWritten: true,
                    next: "/workspace",
                    error: null
                })
            });
            return;
        }

        await route.fulfill({
            status: 405,
            contentType: "application/json",
            body: JSON.stringify({
                error: "setup submissions require POST"
            })
        });
    });

    await page.route("**/api/workspace/files", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(workspaceFiles)
        });
    });

    await page.route("**/api/workspace/reference", async (route) => {
        const body = route.request().postDataJSON() as { relative_path?: string };
        const referenceToken =
            body.relative_path === "src/lib.rs"
                ? "[[workspace:src/lib.rs]]"
                : "[[workspace:src/main.rs]]";

        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify({
                relative_path: body.relative_path ?? "src/main.rs",
                reference_token: referenceToken
            })
        });
    });

    await page.route("**/api/queue/state", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(queueState)
        });
    });

    await page.route("**/api/queue/steering", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify({ ok: true })
        });
    });

    await page.route("**/api/queue/follow-up", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify({ ok: true })
        });
    });

    await page.route("**/api/execution/visibility", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(executionVisibility)
        });
    });

    await page.route("**/api/skills?agent=default", async (route) => {
        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify(skillsInventory)
        });
    });

    await page.route("**/api/skills/toggle", async (route) => {
        const payload = route.request().postDataJSON() as {
            agent_name?: string;
            skill_name?: string;
            enabled?: boolean;
        };

        await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify({
                agent_name: payload.agent_name ?? "default",
                enabled: payload.enabled ? [payload.skill_name ?? "lint-bridge"] : []
            })
        });
    });

    await page.goto("/setup");
    await expect(page.getByRole("heading", { name: "First-launch wizard scaffold" })).toBeVisible();
    await page.screenshot({ path: path.join(artifactDir, "setup.png"), fullPage: true });

    await page.goto("/workspace");
    await expect(page.getByRole("heading", { name: "Files and references" })).toBeVisible();
    await page.getByRole("button", { name: "Reference" }).first().click();
    await expect(page.locator(".reference-chips span").filter({ hasText: "[[workspace:src/main.rs]]" })).toBeVisible();

    const steeringMessage = "Playwright steering smoke message";
    const steeringArea = page.getByPlaceholder("Queue the next-turn steering instruction.");
    await steeringArea.fill(steeringMessage);
    await page.getByRole("button", { name: "Queue steering" }).click();
    await expect(page.getByText(steeringMessage)).toBeVisible();
    await page.screenshot({ path: path.join(artifactDir, "workspace.png"), fullPage: true });

    await page.goto("/skills");
    await expect(page.getByRole("heading", { name: "Global inventory" })).toBeVisible();
    await expect(page.getByRole("button", { name: /research/i })).toBeVisible();
    await expect(page.getByRole("button", { name: /lint-bridge/i })).toBeVisible();
    await page.getByRole("button", { name: /lint-bridge/i }).click();
    await page.getByRole("button", { name: "Enable for default" }).click();
    await expect(page.getByText("Enabled for default")).toBeVisible();
    await page.screenshot({ path: path.join(artifactDir, "skills.png"), fullPage: true });
});
