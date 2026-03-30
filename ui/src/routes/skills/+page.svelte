<script lang="ts">
    import { onMount } from "svelte";

    import { errorMessage, fetchJson } from "$lib/http";

    type SkillCatalogRecord = {
        name: string;
        source_root: string;
        installed_root: string;
        enabled_by_agent_count: number;
        enabled_by_agents: string[];
    };

    let catalog: SkillCatalogRecord[] = [];
    let selectedSkill: SkillCatalogRecord | null = null;
    let loading = true;
    let pageError = "";

    async function loadCatalog() {
        loading = true;
        pageError = "";

        try {
            catalog = await fetchJson<SkillCatalogRecord[]>("/api/skills/catalog");
            selectedSkill = catalog[0] ?? null;
        } catch (error) {
            pageError = errorMessage(error);
        } finally {
            loading = false;
        }
    }

    function enabledByLabel(skill: SkillCatalogRecord): string {
        return skill.enabled_by_agent_count === 1
            ? "Enabled by 1 agent"
            : `Enabled by ${skill.enabled_by_agent_count} agents`;
    }

    onMount(() => {
        void loadCatalog();
    });
</script>

<svelte:head>
    <title>Skills | MatrixClaw</title>
</svelte:head>

<section class="catalog-shell">
    <header class="route-header">
        <p class="section-label">Skills</p>
        <h1>Skills</h1>
        <p class="lead">
            Global skill catalog with per-agent usage counts. Per-agent enablement is managed from
            Agent Detail.
        </p>
        <span class="status-pill">Managed globally, enabled per agent.</span>
    </header>

    {#if pageError}
        <p class="error-copy" role="alert">{pageError}</p>
    {/if}

    {#if loading}
        <p class="state-copy">Loading skills catalog...</p>
    {:else if catalog.length > 0}
        <div class="catalog-grid">
            <aside class="catalog-list-panel">
                <p class="section-label">Installed skills</p>
                <div class="catalog-list">
                    {#each catalog as skill}
                        <button
                            type="button"
                            class:selected={skill.name === selectedSkill?.name}
                            on:click={() => (selectedSkill = skill)}
                        >
                            <div class="catalog-list__copy">
                                <strong>{skill.name}</strong>
                                <small>{skill.source_root}</small>
                            </div>
                            <span>{enabledByLabel(skill)}</span>
                        </button>
                    {/each}
                </div>
            </aside>

            {#if selectedSkill}
                <article class="detail-panel">
                    <p class="section-label">Selected skill</p>
                    <div class="detail-header">
                        <div>
                            <h2>{selectedSkill.name}</h2>
                            <p>
                                Global imports stay immutable. Per-agent enablement is managed from
                                the agent cockpit.
                            </p>
                        </div>
                        <span class="status-pill">{enabledByLabel(selectedSkill)}</span>
                    </div>

                    <dl class="meta-grid">
                        <div>
                            <dt>Source root</dt>
                            <dd>{selectedSkill.source_root}</dd>
                        </div>
                        <div>
                            <dt>Installed root</dt>
                            <dd>{selectedSkill.installed_root}</dd>
                        </div>
                        <div>
                            <dt>Enablement</dt>
                            <dd>{selectedSkill.enabled_by_agent_count}</dd>
                        </div>
                        <div>
                            <dt>Boundary</dt>
                            <dd>Agent Detail</dd>
                        </div>
                    </dl>

                    <div class="binding-cloud">
                        {#if selectedSkill.enabled_by_agents.length > 0}
                            {#each selectedSkill.enabled_by_agents as agent}
                                <span>{agent}</span>
                            {/each}
                        {:else}
                            <p class="state-copy">No agents are currently using this skill.</p>
                        {/if}
                    </div>
                </article>
            {/if}
        </div>
    {:else}
        <p class="state-copy">No installed skills were returned by the catalog.</p>
    {/if}
</section>

<style>
    .catalog-shell {
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
    h2,
    p,
    dl {
        margin: 0;
    }

    h1 {
        color: var(--mc-text);
        font-size: clamp(1.8rem, 3vw, 2.4rem);
        line-height: 1;
    }

    h2 {
        color: var(--mc-text);
        font-size: 1.2rem;
    }

    .lead,
    .state-copy,
    .error-copy,
    .catalog-list__copy small,
    .detail-panel p,
    dd {
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

    .catalog-grid {
        display: grid;
        grid-template-columns: minmax(18rem, 24rem) minmax(0, 1fr);
        gap: 1rem;
        align-items: start;
    }

    .catalog-list-panel,
    .detail-panel {
        padding: 1.1rem;
        border: 1px solid var(--mc-border);
        border-radius: var(--mc-radius-card);
        background: var(--mc-surface);
        box-shadow: 0 10px 18px rgba(30, 36, 48, 0.05);
    }

    .catalog-list {
        display: grid;
        gap: 0.75rem;
        margin-top: 0.9rem;
    }

    .catalog-list button {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 1rem;
        padding: 0.95rem 1rem;
        text-align: left;
        border-radius: var(--mc-radius-card);
        border: 1px solid var(--mc-border);
        background: var(--mc-raised);
        color: inherit;
        cursor: pointer;
    }

    .catalog-list button.selected {
        border-color: rgba(91, 192, 235, 0.4);
        background: rgba(91, 192, 235, 0.08);
    }

    .catalog-list__copy {
        display: grid;
        gap: 0.2rem;
    }

    .catalog-list__copy strong {
        color: var(--mc-text);
        font-size: 1rem;
    }

    .catalog-list__copy small {
        display: block;
    }

    .catalog-list button span,
    .binding-cloud span {
        width: fit-content;
        padding: 0.25rem 0.55rem;
        border-radius: 999px;
        background: rgba(91, 192, 235, 0.12);
        color: var(--mc-text);
        font-size: 0.83rem;
    }

    .detail-header {
        display: flex;
        justify-content: space-between;
        align-items: start;
        gap: 1rem;
        margin: 0.7rem 0 1rem;
    }

    .meta-grid {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 0.75rem;
        margin-top: 1rem;
    }

    .meta-grid div {
        display: grid;
        gap: 0.18rem;
        padding: 0.8rem 0.9rem;
        border: 1px solid var(--mc-border);
        border-radius: var(--mc-radius-input);
        background: var(--mc-raised);
    }

    dt {
        color: var(--mc-text-muted);
        font-size: 0.75rem;
        letter-spacing: 0.08em;
        text-transform: uppercase;
    }

    dd {
        margin: 0;
        color: var(--mc-text);
        word-break: break-word;
    }

    .binding-cloud {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        margin-top: 1rem;
    }

    @media (max-width: 960px) {
        .catalog-grid,
        .meta-grid {
            grid-template-columns: 1fr;
        }

        .detail-header {
            flex-direction: column;
        }
    }
</style>
