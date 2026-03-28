export const LOOPBACK_ORIGIN = "http://127.0.0.1:38495";
export const HEALTH_ROUTE = `${LOOPBACK_ORIGIN}/healthz`;
export const WORKSPACE_ROUTE = `${LOOPBACK_ORIGIN}/workspace`;
export const SETUP_ROUTE = `${LOOPBACK_ORIGIN}/setup`;
export const ATTACH_DELAYS_MS = [0, 300, 900, 1800, 3200];

function escapeHtml(value) {
    return value
        .replaceAll("&", "&amp;")
        .replaceAll("<", "&lt;")
        .replaceAll(">", "&gt;")
        .replaceAll('"', "&quot;")
        .replaceAll("'", "&#39;");
}

export function normalizeHealthPayload(payload) {
    if (!payload || typeof payload !== "object") {
        throw new Error("health payload must be an object");
    }

    const baseUrl =
        typeof payload.baseUrl === "string" && payload.baseUrl.trim().length > 0
            ? payload.baseUrl
            : SETUP_ROUTE;
    const configReady = Boolean(payload.configReady);
    const mode = typeof payload.mode === "string" && payload.mode.trim() ? payload.mode : "setup";

    return {
        mode,
        baseUrl,
        configReady
    };
}

export function resolveProductRoute(payload) {
    const health = normalizeHealthPayload(payload);
    return health.configReady ? WORKSPACE_ROUTE : SETUP_ROUTE;
}

export function buildBootstrapModel({
    phase,
    attempt = 0,
    totalAttempts = ATTACH_DELAYS_MS.length,
    detail = "",
    targetUrl = "",
    canRetry = false,
    health = null
}) {
    const launchReady = phase === "launch-ready";
    const launchFailed = phase === "unavailable";
    const routeKnown = Boolean(targetUrl);

    return {
        phase,
        badge:
            phase === "launch-ready"
                ? "Launching"
                : phase === "unavailable"
                  ? "Waiting For Runtime"
                  : "Bootstrapping",
        title:
            phase === "launch-ready"
                ? `Opening ${health?.configReady ? "workspace" : "setup"} in this window`
                : phase === "unavailable"
                  ? "MatrixClaw runtime is not attached yet"
                  : "Starting the MatrixClaw product shell",
        detail:
            detail ||
            (phase === "launch-ready"
                ? "The desktop shell found the local runtime and is handing the same window to the product surface."
                : phase === "unavailable"
                  ? "The shell stayed in product bootstrap mode because the local runtime did not answer on the loopback boundary."
                  : "The shell is checking the local runtime, deciding whether setup or workspace should open, and keeping startup inside one window."),
        targetUrl,
        canRetry,
        steps: [
            {
                id: "runtime",
                label: "Reach the local runtime boundary",
                status: launchReady || routeKnown ? "done" : launchFailed ? "error" : "active"
            },
            {
                id: "route",
                label: "Decide whether setup or workspace should open",
                status: launchReady ? "done" : routeKnown ? "active" : launchFailed ? "idle" : "idle"
            },
            {
                id: "window",
                label: "Hand the same app window to MatrixClaw",
                status: launchReady ? "active" : launchFailed ? "idle" : "idle"
            }
        ],
        attemptCopy:
            phase === "unavailable"
                ? `Tried ${attempt} of ${totalAttempts} attach attempts.`
                : `Attach attempt ${Math.max(attempt, 1)} of ${totalAttempts}.`
    };
}

export async function fetchHealthSnapshot(fetchImpl = fetch) {
    const response = await fetchImpl(HEALTH_ROUTE);
    if (!response.ok) {
        throw new Error(`unexpected health status: ${response.status}`);
    }

    return normalizeHealthPayload(await response.json());
}

function updateNode(node, value) {
    if (node) {
        node.textContent = value;
    }
}

function renderStepList(node, steps) {
    if (!node) {
        return;
    }

    node.innerHTML = steps
        .map(
            (step) =>
                `<li data-status="${escapeHtml(step.status)}"><span>${escapeHtml(step.label)}</span></li>`
        )
        .join("");
}

export function renderBootstrap(model, documentRef = document) {
    const root = documentRef.getElementById("shell-root");
    if (!root) {
        return model;
    }

    root.dataset.phase = model.phase;
    updateNode(documentRef.getElementById("stage-badge"), model.badge);
    updateNode(documentRef.getElementById("status-title"), model.title);
    updateNode(documentRef.getElementById("status-detail"), model.detail);
    updateNode(documentRef.getElementById("attempt-copy"), model.attemptCopy);
    updateNode(
        documentRef.getElementById("target-copy"),
        model.targetUrl
            ? `Target surface: ${model.targetUrl}`
            : `Target surface: ${SETUP_ROUTE} or ${WORKSPACE_ROUTE}`
    );

    const retryButton = documentRef.getElementById("retry-button");
    if (retryButton instanceof HTMLButtonElement) {
        retryButton.hidden = !model.canRetry;
        retryButton.disabled = !model.canRetry;
    }

    renderStepList(documentRef.getElementById("bootstrap-steps"), model.steps);
    return model;
}

function wait(ms, setTimeoutImpl = window.setTimeout.bind(window)) {
    return new Promise((resolve) => setTimeoutImpl(resolve, ms));
}

function isPreviewMode(locationRef = window.location) {
    const params = new URLSearchParams(locationRef.search);
    return params.get("bootstrap-preview") === "1";
}

export async function attachToProductShell({
    fetchImpl = fetch,
    locationRef = window.location,
    documentRef = document,
    setTimeoutImpl = window.setTimeout.bind(window),
    onRender = (model) => renderBootstrap(model, documentRef)
} = {}) {
    onRender(
        buildBootstrapModel({
            phase: "checking",
            attempt: 1
        })
    );

    let lastError = "The loopback runtime has not responded yet.";

    for (let index = 0; index < ATTACH_DELAYS_MS.length; index += 1) {
        const attempt = index + 1;

        if (ATTACH_DELAYS_MS[index] > 0) {
            await wait(ATTACH_DELAYS_MS[index], setTimeoutImpl);
        }

        onRender(
            buildBootstrapModel({
                phase: "checking",
                attempt,
                detail:
                    attempt === 1
                        ? "Checking the local runtime boundary and deciding whether setup or workspace should open."
                        : `Retrying runtime attach after a short backoff.`
            })
        );

        try {
            const health = await fetchHealthSnapshot(fetchImpl);
            const targetUrl = resolveProductRoute(health);
            const launchModel = buildBootstrapModel({
                phase: "launch-ready",
                attempt,
                health,
                targetUrl
            });
            onRender(launchModel);

            if (typeof window !== "undefined") {
                window.__MATRIXCLAW_DESKTOP_SHELL__ = {
                    ...(window.__MATRIXCLAW_DESKTOP_SHELL__ ?? {}),
                    lastModel: launchModel
                };
            }

            if (!isPreviewMode(locationRef)) {
                await wait(180, setTimeoutImpl);
                locationRef.replace(targetUrl);
            }

            return launchModel;
        } catch (error) {
            lastError = error instanceof Error ? error.message : String(error);
        }
    }

    const failedModel = buildBootstrapModel({
        phase: "unavailable",
        attempt: ATTACH_DELAYS_MS.length,
        detail: `${lastError} Start MatrixClaw runtime support, then retry from this window.`,
        canRetry: true
    });
    onRender(failedModel);

    if (typeof window !== "undefined") {
        window.__MATRIXCLAW_DESKTOP_SHELL__ = {
            ...(window.__MATRIXCLAW_DESKTOP_SHELL__ ?? {}),
            lastModel: failedModel
        };
    }

    return failedModel;
}

function registerRetry(fetchImpl = fetch) {
    const retryButton = document.getElementById("retry-button");
    if (!(retryButton instanceof HTMLButtonElement)) {
        return;
    }

    retryButton.addEventListener("click", () => {
        retryButton.disabled = true;
        void attachToProductShell({ fetchImpl }).finally(() => {
            retryButton.disabled = false;
        });
    });
}

if (typeof window !== "undefined" && typeof document !== "undefined") {
    registerRetry();
    void attachToProductShell();
}
