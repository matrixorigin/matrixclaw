<script lang="ts">
    import "$lib/theme/product-shell.css";
    import { page } from "$app/stores";
    import {
        appShellNav,
        describeRoute,
        isActiveRoute,
        shellSignals
    } from "$lib/app-shell/state";

    const brand = "MatrixClaw";
</script>

<svelte:head>
    <title>MatrixClaw</title>
</svelte:head>

<div class="product-shell">
    <div class="product-shell__frame">
        <header class="shell-topbar">
            <div class="window-controls" aria-hidden="true">
                <span class="danger"></span>
                <span class="warn"></span>
                <span class="ok"></span>
            </div>

            <div class="shell-brand">
                <p class="eyebrow">Product shell</p>
                <strong>{brand}</strong>
                <span>Workspace, agents, skills, MCP, and gateway.</span>
            </div>

            <div class="shell-route">
                <span>Current route</span>
                <strong>{describeRoute($page.url.pathname)}</strong>
            </div>
        </header>

        <div class="shell-body">
            <aside class="sidebar">
                <div class="brand-block">
                    <p class="eyebrow">Unified surface</p>
                    <h1>{brand}</h1>
                    <p class="brand-copy">
                        A light desktop shell for workspace execution, agent control, skills
                        management, MCP inspection, and gateway routing.
                    </p>
                </div>

                <nav aria-label="Primary">
                    {#each appShellNav as route}
                        <a
                            class:active={isActiveRoute($page.url.pathname, route.href)}
                            href={route.href}
                        >
                            <div class="nav-header">
                                <strong>{route.label}</strong>
                                <span>{route.shortcut}</span>
                            </div>
                            <small>{route.caption}</small>
                        </a>
                    {/each}
                </nav>

                <section class="signal-panel" aria-label="Shell status">
                    <p class="panel-label">Shell state</p>
                    <div class="signal-grid">
                        {#each shellSignals as signal}
                            <article class={`signal-card ${signal.tone}`}>
                                <span>{signal.label}</span>
                                <strong>{signal.value}</strong>
                            </article>
                        {/each}
                    </div>
                </section>
            </aside>

            <section class="content-column">
                <div class="content-header">
                    <div>
                        <p class="panel-label">Current surface</p>
                        <h2>{describeRoute($page.url.pathname)}</h2>
                    </div>
                    <p class="content-copy">
                        The product shell keeps the active route visible while the inner surface
                        owns its own workflow and data.
                    </p>
                </div>

                <main class="viewport">
                    <slot />
                </main>
            </section>
        </div>
    </div>
</div>

<style>
    :global(body) {
        margin: 0;
        min-height: 100vh;
        background:
            radial-gradient(circle at top left, rgba(99, 89, 243, 0.08), transparent 26%),
            radial-gradient(circle at top right, rgba(245, 72, 64, 0.05), transparent 22%),
            linear-gradient(180deg, var(--mc-bg) 0%, #f3f4f8 54%, #eceef4 100%);
        color: var(--mc-text);
        font-family:
            "Space Grotesk",
            "IBM Plex Sans",
            "Avenir Next",
            sans-serif;
    }

    :global(*) {
        box-sizing: border-box;
    }

    :global(a) {
        color: inherit;
        text-decoration: none;
    }

    .product-shell {
        min-height: 100vh;
        padding: 1rem;
    }

    .product-shell__frame {
        min-height: calc(100vh - 2rem);
        border: 1px solid var(--mc-border);
        border-radius: var(--mc-radius-panel);
        background:
            linear-gradient(180deg, rgba(255, 255, 255, 0.92), rgba(248, 249, 252, 0.96));
        box-shadow:
            0 24px 60px rgba(44, 52, 72, 0.12),
            inset 0 1px 0 rgba(255, 255, 255, 0.75);
        overflow: hidden;
    }

    .shell-topbar {
        display: grid;
        grid-template-columns: auto 1fr auto;
        gap: 1rem;
        align-items: center;
        padding: 0.95rem 1.2rem;
        border-bottom: 1px solid var(--mc-border);
        background: rgba(255, 255, 255, 0.82);
        backdrop-filter: blur(14px);
    }

    .window-controls {
        display: flex;
        gap: 0.45rem;
    }

    .window-controls span {
        width: 0.78rem;
        height: 0.78rem;
        border-radius: 999px;
        box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.28);
    }

    .danger {
        background: var(--mc-accent);
    }

    .warn {
        background: var(--mc-warning);
    }

    .ok {
        background: var(--mc-success);
    }

    .shell-brand,
    .shell-route {
        display: grid;
        gap: 0.15rem;
    }

    .shell-brand strong,
    .shell-route strong {
        color: var(--mc-text);
    }

    .shell-route {
        justify-items: end;
    }

    .shell-brand span,
    .shell-route span,
    .brand-copy,
    .content-copy,
    nav small,
    .panel-label {
        color: var(--mc-text-secondary);
    }

    .eyebrow {
        margin: 0 0 0.35rem;
        color: var(--mc-primary);
        font-size: 0.78rem;
        letter-spacing: 0.18em;
        text-transform: uppercase;
    }

    h1 {
        margin: 0;
        font-size: clamp(2rem, 3vw, 2.8rem);
        line-height: 0.95;
        color: var(--mc-text);
    }

    .shell-body {
        display: grid;
        grid-template-columns: minmax(18rem, 22rem) 1fr;
        min-height: calc(100vh - 6.25rem);
    }

    .sidebar {
        display: grid;
        gap: 1.25rem;
        padding: 1.25rem;
        border-right: 1px solid var(--mc-border);
        background:
            radial-gradient(circle at top, rgba(99, 89, 243, 0.06), transparent 30%),
            linear-gradient(180deg, rgba(248, 248, 250, 0.96), rgba(243, 244, 247, 0.96));
    }

    .brand-block {
        display: grid;
        gap: 0.7rem;
        padding: 1.1rem;
        border: 1px solid var(--mc-border);
        border-radius: var(--mc-radius-card);
        background: var(--mc-surface);
        box-shadow: 0 10px 20px rgba(30, 36, 48, 0.05);
    }

    nav {
        display: grid;
        gap: 0.8rem;
    }

    nav a {
        width: 100%;
        text-align: left;
        display: grid;
        gap: 0.45rem;
        padding: 0.95rem 1rem;
        border: 1px solid var(--mc-border);
        border-radius: var(--mc-radius-button);
        background: var(--mc-surface);
        box-shadow: 0 6px 12px rgba(18, 22, 33, 0.03);
        color: inherit;
        transition:
            transform 180ms ease,
            border-color 180ms ease,
            background 180ms ease,
            box-shadow 180ms ease;
    }

    nav a.active,
    nav a:hover {
        transform: translateY(-1px);
        border-color: var(--mc-border-strong);
        background: var(--mc-hover);
        box-shadow: 0 14px 24px rgba(18, 22, 33, 0.08);
    }

    .nav-header {
        display: flex;
        justify-content: space-between;
        gap: 1rem;
        align-items: center;
    }

    .nav-header strong {
        font-size: 1rem;
        color: var(--mc-text);
    }

    .nav-header span {
        color: var(--mc-text-muted);
        font-size: 0.76rem;
        letter-spacing: 0.08em;
        text-transform: uppercase;
    }

    .signal-panel {
        display: grid;
        gap: 0.8rem;
    }

    .panel-label {
        margin: 0;
        font-size: 0.78rem;
        letter-spacing: 0.14em;
        text-transform: uppercase;
    }

    .signal-grid {
        display: grid;
        gap: 0.75rem;
    }

    .signal-card {
        display: grid;
        gap: 0.25rem;
        padding: 0.9rem 1rem;
        border-radius: var(--mc-radius-card);
        border: 1px solid var(--mc-border);
        background: var(--mc-raised);
    }

    .signal-card span {
        color: var(--mc-text-muted);
        font-size: 0.8rem;
        text-transform: uppercase;
        letter-spacing: 0.1em;
    }

    .signal-card strong {
        color: var(--mc-text);
    }

    .signal-card.accent {
        border-color: rgba(99, 89, 243, 0.2);
    }

    .signal-card.warn {
        border-color: rgba(245, 199, 43, 0.36);
    }

    .content-column {
        display: grid;
        grid-template-rows: auto 1fr;
        min-width: 0;
        background: linear-gradient(180deg, rgba(255, 255, 255, 0.24), transparent);
    }

    .content-header {
        display: flex;
        flex-wrap: wrap;
        gap: 1rem;
        justify-content: space-between;
        align-items: end;
        padding: 1.25rem 1.35rem 0;
    }

    h2 {
        margin: 0.2rem 0 0;
        font-size: clamp(1.35rem, 2.3vw, 2rem);
        color: var(--mc-text);
    }

    .content-copy {
        max-width: 32rem;
        line-height: 1.6;
    }

    .viewport {
        min-height: 0;
        padding: 1.25rem 1.35rem 1.35rem;
        overflow: auto;
    }

    @media (max-width: 920px) {
        .shell-body {
            grid-template-columns: 1fr;
        }

        .sidebar {
            border-right: 0;
            border-bottom: 1px solid var(--mc-border);
        }

        .shell-topbar {
            grid-template-columns: 1fr;
        }

        .shell-route {
            justify-items: start;
        }
    }

    @media (max-width: 720px) {
        .product-shell {
            padding: 0.65rem;
        }

        .product-shell__frame {
            min-height: calc(100vh - 1.3rem);
            border-radius: var(--mc-radius-card);
        }

        .content-header,
        .viewport,
        .sidebar,
        .shell-topbar {
            padding-left: 1rem;
            padding-right: 1rem;
        }
    }
</style>
