import test from "node:test";
import assert from "node:assert/strict";

import {
    ATTACH_DELAYS_MS,
    SETUP_ROUTE,
    WORKSPACE_ROUTE,
    buildBootstrapModel,
    normalizeHealthPayload,
    resolveProductRoute
} from "./launcher.js";

test("normalizeHealthPayload preserves valid health fields", () => {
    assert.deepEqual(
        normalizeHealthPayload({
            mode: "setup",
            baseUrl: "http://127.0.0.1:38495/setup",
            configReady: true
        }),
        {
            mode: "setup",
            baseUrl: "http://127.0.0.1:38495/setup",
            configReady: true
        }
    );
});

test("normalizeHealthPayload falls back to setup defaults", () => {
    assert.deepEqual(normalizeHealthPayload({}), {
        mode: "setup",
        baseUrl: SETUP_ROUTE,
        configReady: false
    });
});

test("resolveProductRoute opens workspace when config is ready", () => {
    assert.equal(
        resolveProductRoute({
            baseUrl: "http://127.0.0.1:38495/setup",
            configReady: true
        }),
        WORKSPACE_ROUTE
    );
});

test("resolveProductRoute opens setup when config is missing", () => {
    assert.equal(
        resolveProductRoute({
            baseUrl: "http://127.0.0.1:38495/setup",
            configReady: false
        }),
        SETUP_ROUTE
    );
});

test("buildBootstrapModel marks launch-ready progress inside one window", () => {
    const model = buildBootstrapModel({
        phase: "launch-ready",
        attempt: 2,
        totalAttempts: ATTACH_DELAYS_MS.length,
        health: {
            configReady: true
        },
        targetUrl: WORKSPACE_ROUTE
    });

    assert.equal(model.badge, "Launching");
    assert.equal(model.steps[0].status, "done");
    assert.equal(model.steps[1].status, "done");
    assert.equal(model.steps[2].status, "active");
    assert.equal(model.targetUrl, WORKSPACE_ROUTE);
});

test("buildBootstrapModel surfaces retry state when runtime is unavailable", () => {
    const model = buildBootstrapModel({
        phase: "unavailable",
        attempt: ATTACH_DELAYS_MS.length,
        totalAttempts: ATTACH_DELAYS_MS.length,
        canRetry: true
    });

    assert.equal(model.badge, "Waiting For Runtime");
    assert.equal(model.canRetry, true);
    assert.match(model.attemptCopy, new RegExp(`${ATTACH_DELAYS_MS.length}`));
    assert.equal(model.steps[0].status, "error");
});
