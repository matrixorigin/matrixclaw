<script lang="ts">
    import { page } from "$app/stores";
    import { shellRoutes } from "$lib/shell";

    const brand = "MatrixClaw";
</script>

<svelte:head>
    <title>MatrixClaw</title>
</svelte:head>

<div class="app-shell">
    <header class="topbar">
        <div>
            <p class="eyebrow">Local-first agent runtime</p>
            <h1>{brand}</h1>
        </div>
        <nav aria-label="Primary">
            {#each shellRoutes as route}
                <a
                    class:active={$page.url.pathname.startsWith(route.href)}
                    href={route.href}
                >
                    <span>{route.label}</span>
                    <small>{route.description}</small>
                </a>
            {/each}
        </nav>
    </header>

    <main>
        <slot />
    </main>
</div>

<style>
    :global(body) {
        margin: 0;
        min-height: 100vh;
        background:
            radial-gradient(circle at top, rgba(251, 191, 36, 0.12), transparent 34%),
            linear-gradient(180deg, #0f172a 0%, #111827 42%, #030712 100%);
        color: #e5eef7;
        font-family:
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

    .app-shell {
        min-height: 100vh;
        padding: 1.5rem;
    }

    .topbar {
        display: flex;
        flex-wrap: wrap;
        gap: 1.5rem;
        justify-content: space-between;
        align-items: flex-start;
        margin: 0 auto 1.5rem;
        max-width: 84rem;
    }

    .eyebrow {
        margin: 0 0 0.35rem;
        color: #fbbf24;
        font-size: 0.78rem;
        letter-spacing: 0.18em;
        text-transform: uppercase;
    }

    h1 {
        margin: 0;
        font-size: clamp(2rem, 4vw, 3.2rem);
        line-height: 0.95;
    }

    nav {
        display: grid;
        gap: 0.75rem;
        min-width: min(100%, 24rem);
    }

    nav a {
        display: grid;
        gap: 0.15rem;
        padding: 0.85rem 1rem;
        border: 1px solid rgba(148, 163, 184, 0.2);
        border-radius: 1rem;
        background: rgba(15, 23, 42, 0.55);
        transition:
            transform 160ms ease,
            border-color 160ms ease,
            background 160ms ease;
    }

    nav a.active,
    nav a:hover {
        transform: translateY(-1px);
        border-color: rgba(251, 191, 36, 0.55);
        background: rgba(17, 24, 39, 0.9);
    }

    nav small {
        color: #94a3b8;
        line-height: 1.4;
    }

    main {
        margin: 0 auto;
        max-width: 84rem;
    }

    @media (max-width: 720px) {
        .app-shell {
            padding: 1rem;
        }

        nav {
            min-width: 100%;
        }
    }
</style>
