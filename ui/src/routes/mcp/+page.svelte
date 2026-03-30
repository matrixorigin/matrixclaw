<script lang="ts">
    import { onMount } from "svelte";

    import { fetchMcpCatalog, type McpCatalogRecord } from "$lib/catalogs/mcp";

    let catalog: McpCatalogRecord[] = [];
    let loading = true;
    let pageError = "";

    async function loadCatalog() {
        loading = true;
        pageError = "";

        try {
            catalog = await fetchMcpCatalog();
        } catch (error) {
            pageError = error instanceof Error ? error.message : String(error);
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        void loadCatalog();
    });
</script>

<svelte:head>
    <title>MCP | MatrixClaw</title>
</svelte:head>

<section class="route-shell">
    <header class="route-header">
        <p class="section-label">MCP</p>
        <h1>MCP</h1>
        <p class="lead">Global MCP catalog and live usage counts.</p>
    </header>

    {#if pageError}
        <p class="error-copy" role="alert">{pageError}</p>
    {/if}

    {#if loading}
        <p class="state-copy">Loading MCP catalog...</p>
    {:else if catalog.length > 0}
        <div class="catalog-list">
            {#each catalog as item}
                <article class="catalog-card">
                    <div>
                        <strong>{item.name}</strong>
                        <p>{item.health}</p>
                    </div>
                    <span>Enabled by {item.enabled_by_agent_count} agents</span>
                </article>
            {/each}
        </div>
    {:else}
        <p class="state-copy">No MCP servers were returned.</p>
    {/if}
</section>

<style>
    .route-shell {
        display: grid;
        gap: 1rem;
    }

    .route-header {
        display: grid;
        gap: 0.5rem;
    }

    .section-label {
        margin: 0;
        color: var(--mc-primary);
        font-size: 0.78rem;
        letter-spacing: 0.16em;
        text-transform: uppercase;
    }

    h1,
    p {
        margin: 0;
    }

    h1 {
        color: var(--mc-text);
        font-size: clamp(1.8rem, 3vw, 2.4rem);
        line-height: 1;
    }

    .lead,
    .state-copy,
    .error-copy,
    .catalog-card p {
        color: var(--mc-text-secondary);
        line-height: 1.55;
    }

    .catalog-list {
        display: grid;
        gap: 0.75rem;
    }

    .catalog-card {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        padding: 1rem 1.1rem;
        border: 1px solid var(--mc-border);
        border-radius: var(--mc-radius-card);
        background: var(--mc-surface);
        box-shadow: 0 10px 18px rgba(30, 36, 48, 0.05);
    }

    .catalog-card strong {
        color: var(--mc-text);
    }

    .catalog-card span {
        color: var(--mc-text);
        font-size: 0.88rem;
    }
</style>
