<script lang="ts">
    import { onMount } from "svelte";

    import { fetchGatewayCatalog, type GatewayCatalogRecord } from "$lib/catalogs/gateway";

    let catalog: GatewayCatalogRecord[] = [];
    let loading = true;
    let pageError = "";

    async function loadCatalog() {
        loading = true;
        pageError = "";

        try {
            catalog = await fetchGatewayCatalog();
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
    <title>Gateway | MatrixClaw</title>
</svelte:head>

<section class="route-shell">
    <header class="route-header">
        <p class="section-label">Gateway</p>
        <h1>Gateway</h1>
        <p class="lead">Global messaging gateway catalog and live usage counts.</p>
        <span class="status-pill">Managed centrally, enabled per agent.</span>
    </header>

    {#if pageError}
        <p class="error-copy" role="alert">{pageError}</p>
    {/if}

    {#if loading}
        <p class="state-copy">Loading gateway catalog...</p>
    {:else if catalog.length > 0}
        <div class="catalog-list">
            {#each catalog as item}
                <article class="catalog-card">
                    <div class="catalog-card__copy">
                        <strong>{item.name}</strong>
                        <p>{item.health}</p>
                    </div>
                    <span class="status-pill">{item.enabled_by_agent_count === 1 ? "Enabled by 1 agent" : `Enabled by ${item.enabled_by_agent_count} agents`}</span>
                </article>
            {/each}
        </div>
    {:else}
        <p class="state-copy">No gateways were returned.</p>
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

    .status-pill {
        width: fit-content;
        padding: 0.34rem 0.62rem;
        border-radius: 999px;
        border: 1px solid var(--mc-border);
        background: rgba(91, 192, 235, 0.12);
        color: var(--mc-text);
        font-size: 0.85rem;
        font-weight: 500;
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

    .catalog-card__copy {
        display: grid;
        gap: 0.2rem;
    }

    .catalog-card strong {
        color: var(--mc-text);
    }

    .catalog-card span {
        color: var(--mc-text);
        font-size: 0.88rem;
        white-space: nowrap;
    }
</style>
