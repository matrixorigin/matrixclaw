<script lang="ts">
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

<div class="desktop-shell">
    <div class="desktop-frame">
        <header class="windowbar">
            <div class="window-controls" aria-hidden="true">
                <span class="danger"></span>
                <span class="warn"></span>
                <span class="ok"></span>
            </div>

            <div class="window-meta">
                <p class="eyebrow">Tauri desktop shell</p>
                <strong>{brand}</strong>
            </div>

            <div class="window-route">
                <span>Surface</span>
                <strong>{describeRoute($page.url.pathname)}</strong>
            </div>
        </header>

        <div class="frame-body">
            <aside class="sidebar">
                <div class="brand-block">
                    <p class="eyebrow">Agent runtime</p>
                    <h1>{brand}</h1>
                    <p class="brand-copy">
                        A desktop shell for setup, workspace handoff, and runtime visibility.
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
                        <p class="panel-label">Current route</p>
                        <h2>{describeRoute($page.url.pathname)}</h2>
                    </div>
                    <p class="content-copy">
                        Desktop app framing replaces the preview-site shell while preserving route
                        ownership boundaries.
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
    :global(:root) {
        color-scheme: dark;
        --shell-bg: #09111f;
        --shell-panel: rgba(9, 18, 33, 0.84);
        --shell-panel-strong: rgba(14, 26, 48, 0.92);
        --shell-border: rgba(148, 163, 184, 0.16);
        --shell-copy: #b5c4d8;
        --shell-bright: #eef4ff;
        --shell-accent: #7dd3fc;
        --shell-accent-2: #f59e0b;
        --shell-danger: #fb7185;
    }

    :global(body) {
        margin: 0;
        min-height: 100vh;
        background:
            radial-gradient(circle at top left, rgba(125, 211, 252, 0.18), transparent 24%),
            radial-gradient(circle at top right, rgba(245, 158, 11, 0.16), transparent 22%),
            linear-gradient(180deg, #060b14 0%, #08111f 48%, #030712 100%);
        color: var(--shell-bright);
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

    .desktop-shell {
        min-height: 100vh;
        padding: 1rem;
    }

    .desktop-frame {
        min-height: calc(100vh - 2rem);
        border: 1px solid var(--shell-border);
        border-radius: 1.6rem;
        background:
            linear-gradient(180deg, rgba(255, 255, 255, 0.02), transparent 16%),
            rgba(3, 8, 18, 0.72);
        box-shadow:
            0 30px 80px rgba(2, 6, 23, 0.55),
            inset 0 1px 0 rgba(255, 255, 255, 0.04);
        backdrop-filter: blur(18px);
        overflow: hidden;
    }

    .windowbar {
        display: grid;
        grid-template-columns: auto 1fr auto;
        gap: 1rem;
        align-items: center;
        padding: 0.9rem 1.15rem;
        border-bottom: 1px solid var(--shell-border);
        background: rgba(6, 13, 24, 0.86);
    }

    .window-controls {
        display: flex;
        gap: 0.45rem;
    }

    .window-controls span {
        width: 0.78rem;
        height: 0.78rem;
        border-radius: 999px;
        box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.22);
    }

    .danger {
        background: #fb7185;
    }

    .warn {
        background: #f59e0b;
    }

    .ok {
        background: #34d399;
    }

    .window-meta,
    .window-route {
        display: grid;
        gap: 0.15rem;
    }

    .eyebrow {
        margin: 0 0 0.35rem;
        color: var(--shell-accent);
        font-size: 0.78rem;
        letter-spacing: 0.18em;
        text-transform: uppercase;
    }

    .window-meta strong,
    .window-route strong {
        color: var(--shell-bright);
    }

    .window-route {
        justify-items: end;
    }

    .window-route span,
    .brand-copy,
    .content-copy,
    nav small {
        color: var(--shell-copy);
    }

    h1 {
        margin: 0;
        font-size: clamp(2rem, 3vw, 2.8rem);
        line-height: 0.95;
    }

    .frame-body {
        display: grid;
        grid-template-columns: minmax(17rem, 20rem) 1fr;
        min-height: calc(100vh - 6.25rem);
    }

    .sidebar {
        display: grid;
        gap: 1.25rem;
        padding: 1.25rem;
        border-right: 1px solid var(--shell-border);
        background:
            radial-gradient(circle at top, rgba(125, 211, 252, 0.08), transparent 30%),
            rgba(7, 13, 24, 0.9);
    }

    .brand-block {
        display: grid;
        gap: 0.7rem;
        padding: 1.1rem;
        border: 1px solid rgba(125, 211, 252, 0.14);
        border-radius: 1.25rem;
        background: rgba(10, 19, 35, 0.72);
    }

    nav {
        display: grid;
        gap: 0.8rem;
    }

    nav a {
        display: grid;
        gap: 0.45rem;
        padding: 0.95rem 1rem;
        border: 1px solid rgba(148, 163, 184, 0.14);
        border-radius: 1.1rem;
        background: rgba(10, 18, 32, 0.72);
        transition:
            transform 180ms ease,
            border-color 180ms ease,
            background 180ms ease,
            box-shadow 180ms ease;
    }

    nav a.active,
    nav a:hover {
        transform: translateX(0.15rem);
        border-color: rgba(125, 211, 252, 0.35);
        background: rgba(17, 30, 51, 0.96);
        box-shadow: 0 16px 32px rgba(2, 6, 23, 0.2);
    }

    .nav-header {
        display: flex;
        justify-content: space-between;
        gap: 1rem;
        align-items: center;
    }

    .nav-header strong {
        font-size: 1rem;
    }

    .nav-header span {
        color: rgba(181, 196, 216, 0.78);
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
        color: rgba(181, 196, 216, 0.82);
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
        border-radius: 1rem;
        border: 1px solid rgba(148, 163, 184, 0.12);
        background: rgba(8, 15, 28, 0.8);
    }

    .signal-card span {
        color: var(--shell-copy);
        font-size: 0.8rem;
        text-transform: uppercase;
        letter-spacing: 0.1em;
    }

    .signal-card.accent {
        border-color: rgba(125, 211, 252, 0.22);
    }

    .signal-card.warn {
        border-color: rgba(245, 158, 11, 0.22);
    }

    .content-column {
        display: grid;
        grid-template-rows: auto 1fr;
        min-width: 0;
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
        .frame-body {
            grid-template-columns: 1fr;
        }

        .sidebar {
            border-right: 0;
            border-bottom: 1px solid var(--shell-border);
        }

        .windowbar {
            grid-template-columns: 1fr;
        }

        .window-route {
            justify-items: start;
        }
    }

    @media (max-width: 720px) {
        .desktop-shell {
            padding: 0.65rem;
        }

        .desktop-frame {
            min-height: calc(100vh - 1.3rem);
            border-radius: 1.2rem;
        }

        .content-header,
        .viewport,
        .sidebar,
        .windowbar {
            padding-left: 1rem;
            padding-right: 1rem;
        }
    }
</style>
