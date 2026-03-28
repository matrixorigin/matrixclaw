<script lang="ts">
    import { goto } from "$app/navigation";
    import { bootNotes } from "$lib/app-shell/state";
    import { errorMessage, fetchJson } from "$lib/http";
    import { onMount } from "svelte";

    type HealthSnapshot = {
        mode: string;
        baseUrl: string;
        configReady: boolean;
    };

    let routeMessage = "Checking local runtime state.";
    let routeHint = "The desktop shell is waiting for the Rust runtime to answer `/healthz`.";
    let routeTarget = "/setup";
    let routeLabel = "Open setup";
    let health: HealthSnapshot | null = null;
    let launchError = "";

    onMount(async () => {
        try {
            health = await fetchJson<HealthSnapshot>("/healthz");
            routeMessage = health.configReady
                ? "Configuration is present. Handing off to the workspace shell."
                : "Configuration is missing. Handing off to onboarding.";
            routeHint = health.configReady
                ? "The runtime reported a ready config, so the app can skip first-launch setup."
                : "The runtime needs provider, workspace, auth, and execution defaults before the workspace can open.";
            routeTarget = health.configReady ? "/workspace" : "/setup";
            routeLabel = health.configReady ? "Open workspace" : "Open setup";
            await goto(routeTarget);
        } catch (error) {
            launchError = errorMessage(error);
            routeMessage = "Loopback health is unavailable.";
            routeHint = "Manual routes remain available while the runtime is offline.";
        }
    });
</script>

<section class="launchpad">
    <div class="hero-card">
        <div class="hero-copy">
            <p class="tag">Desktop handoff</p>
            <h2>Launch surface for the Tauri shell</h2>
            <p>
                MatrixClaw now boots like a desktop app: probe runtime state, route to onboarding
                when config is missing, and hand off directly to the working shell once the runtime
                is ready.
            </p>
        </div>

        <div class="pulse-card">
            <span class="pulse"></span>
            <div>
                <strong>{routeMessage}</strong>
                <p>{routeHint}</p>
            </div>
        </div>

        <div class="launch-actions">
            <a href={routeTarget}>{routeLabel}</a>
            <a class="secondary" href="/setup">Review onboarding</a>
            <a class="secondary" href="/workspace">Open workspace</a>
        </div>
    </div>

    <div class="grid">
        <article class="status-card">
            <p class="card-label">Runtime probe</p>
            <strong>{health ? health.mode : "Waiting for response"}</strong>
            <p>{health ? `Base URL: ${health.baseUrl}` : "A live runtime response will appear here."}</p>
            {#if launchError}
                <p class="error-copy">{launchError}</p>
            {/if}
        </article>

        <article class="status-card">
            <p class="card-label">Launch route</p>
            <strong>{routeTarget}</strong>
            <p>
                The root page remains the launch switchboard, but the visual shell now matches the
                desktop product instead of a preview scaffold.
            </p>
        </article>

        <article class="notes-card">
            <p class="card-label">Boot notes</p>
            <ul>
                {#each bootNotes as note}
                    <li>
                        <strong>{note.label}</strong>
                        <p>{note.detail}</p>
                    </li>
                {/each}
            </ul>
        </article>
    </div>
</section>

<style>
    .launchpad {
        display: grid;
        gap: 1.15rem;
    }

    .hero-card {
        display: grid;
        gap: 1.2rem;
        padding: 1.5rem;
        border: 1px solid rgba(125, 211, 252, 0.14);
        border-radius: 1.45rem;
        background:
            radial-gradient(circle at top right, rgba(125, 211, 252, 0.12), transparent 26%),
            linear-gradient(135deg, rgba(10, 20, 38, 0.94), rgba(8, 14, 24, 0.86));
        box-shadow: 0 20px 60px rgba(2, 6, 23, 0.28);
    }

    .tag {
        margin: 0 0 0.5rem;
        color: #7dd3fc;
        text-transform: uppercase;
        letter-spacing: 0.16em;
        font-size: 0.78rem;
    }

    h2 {
        margin: 0 0 0.6rem;
        font-size: clamp(1.8rem, 4vw, 3.2rem);
        line-height: 0.95;
    }

    p,
    li {
        color: #b5c4d8;
        line-height: 1.6;
    }

    .pulse-card,
    article {
        padding: 1.1rem 1.2rem;
        border: 1px solid rgba(148, 163, 184, 0.14);
        border-radius: 1.2rem;
        background: rgba(7, 14, 26, 0.72);
    }

    .pulse-card {
        display: grid;
        grid-template-columns: auto 1fr;
        gap: 0.9rem;
        align-items: start;
    }

    .pulse {
        width: 0.9rem;
        height: 0.9rem;
        margin-top: 0.25rem;
        border-radius: 999px;
        background: #34d399;
        box-shadow:
            0 0 0 0 rgba(52, 211, 153, 0.45),
            0 0 0 0 rgba(52, 211, 153, 0.25);
        animation: pulse 1.8s ease-out infinite;
    }

    .pulse-card strong,
    article strong {
        display: block;
        margin-bottom: 0.3rem;
        color: #eef4ff;
        font-size: 1.05rem;
    }

    .launch-actions {
        display: flex;
        flex-wrap: wrap;
        gap: 0.8rem;
    }

    a {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        padding: 0.8rem 1rem;
        border-radius: 999px;
        border: 1px solid rgba(125, 211, 252, 0.18);
        background: rgba(125, 211, 252, 0.12);
        color: #eef4ff;
        font-weight: 600;
    }

    a.secondary {
        background: rgba(255, 255, 255, 0.03);
    }

    .grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(16.5rem, 1fr));
        gap: 1rem;
    }

    .card-label {
        margin: 0 0 0.6rem;
        color: #7dd3fc;
        font-size: 0.78rem;
        text-transform: uppercase;
        letter-spacing: 0.16em;
    }

    ul {
        margin: 0;
        padding: 0;
        list-style: none;
        display: grid;
        gap: 0.8rem;
    }

    .notes-card li {
        padding-top: 0.8rem;
        border-top: 1px solid rgba(148, 163, 184, 0.12);
    }

    .notes-card li:first-child {
        padding-top: 0;
        border-top: 0;
    }

    .notes-card p,
    .status-card p {
        margin: 0;
    }

    .error-copy {
        color: #fca5a5;
    }

    @keyframes pulse {
        0% {
            box-shadow:
                0 0 0 0 rgba(52, 211, 153, 0.45),
                0 0 0 0 rgba(52, 211, 153, 0.25);
        }

        70% {
            box-shadow:
                0 0 0 12px rgba(52, 211, 153, 0),
                0 0 0 22px rgba(52, 211, 153, 0);
        }

        100% {
            box-shadow:
                0 0 0 0 rgba(52, 211, 153, 0),
                0 0 0 0 rgba(52, 211, 153, 0);
        }
    }

    @media (max-width: 720px) {
        .hero-card {
            padding: 1.15rem;
        }

        .pulse-card {
            grid-template-columns: 1fr;
        }
    }
</style>
