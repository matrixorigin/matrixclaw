<script lang="ts">
    import { page } from "$app/stores";
    import { onMount } from "svelte";

    import { fetchAgent, type AgentSummary } from "$lib/agents";

    let agentId = "";
    let detail: AgentSummary | null = null;
    let loading = true;
    let pageError = "";

    async function loadDetail(currentAgentId: string) {
        if (!currentAgentId) {
            detail = null;
            loading = false;
            pageError = "agent id is required";
            return;
        }

        loading = true;
        pageError = "";

        try {
            detail = await fetchAgent(currentAgentId);
        } catch (error) {
            detail = null;
            pageError = error instanceof Error ? error.message : String(error);
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        const unsubscribe = page.subscribe((value) => {
            agentId = value.params.agentId ?? "";
            void loadDetail(agentId);
        });

        return unsubscribe;
    });
</script>

<svelte:head>
    <title>{detail?.title ?? "Agent Detail"} | MatrixClaw</title>
</svelte:head>

<section class="route-shell">
    <header class="route-header">
        <p class="section-label">Agent Detail</p>
        <h1>Agent Detail</h1>
        {#if detail}
            <p class="lead">{detail.title}</p>
            <span class="status-pill">Agent {detail.agent_name} · {detail.binding_count} bindings</span>
        {:else}
            <p class="lead">Per-agent crown job, memory, and bindings live here.</p>
        {/if}
    </header>

    {#if pageError}
        <p class="error-copy" role="alert">{pageError}</p>
    {/if}

    {#if loading}
        <p class="state-copy">Loading agent detail...</p>
    {:else if detail}
        <div class="detail-grid">
            <article class="detail-card detail-card--summary">
                <p class="section-label">Identity</p>
                <div class="summary-row">
                    <div>
                        <h2>{detail.title}</h2>
                        <p>Agent name: {detail.agent_name}</p>
                    </div>
                    <span class="status-pill">Memory {detail.memory_signal_count}</span>
                </div>
                <dl class="meta-list">
                    <div>
                        <dt>Agent</dt>
                        <dd>{detail.agent_name}</dd>
                    </div>
                    <div>
                        <dt>Memory signals</dt>
                        <dd>{detail.memory_signal_count}</dd>
                    </div>
                    <div>
                        <dt>Bindings</dt>
                        <dd>{detail.binding_count}</dd>
                    </div>
                </dl>
            </article>

            <article class="detail-card">
                <p class="section-label">Crown Job</p>
                <h2>Crown Job</h2>
                <p>{detail.crown_job}</p>
            </article>

            <article class="detail-card">
                <p class="section-label">Memory</p>
                <h2>Memory</h2>
                <p>{detail.memory_summary}</p>
            </article>

            <article class="detail-card">
                <p class="section-label">Enabled Skills</p>
                <h2>Enabled Skills</h2>
                <div class="binding-cloud">
                    {#if detail.enabled_skills.length > 0}
                        {#each detail.enabled_skills as skill}
                            <span>{skill}</span>
                        {/each}
                    {:else}
                        <p class="state-copy">No skills are enabled for this agent.</p>
                    {/if}
                </div>
            </article>

            <article class="detail-card">
                <p class="section-label">Enabled MCP Servers</p>
                <h2>Enabled MCP Servers</h2>
                <div class="binding-cloud">
                    {#if detail.enabled_mcp_servers.length > 0}
                        {#each detail.enabled_mcp_servers as server}
                            <span>{server}</span>
                        {/each}
                    {:else}
                        <p class="state-copy">No MCP servers are enabled for this agent.</p>
                    {/if}
                </div>
            </article>

            <article class="detail-card">
                <p class="section-label">Enabled Gateways</p>
                <h2>Enabled Gateways</h2>
                <div class="binding-cloud">
                    {#if detail.enabled_gateways.length > 0}
                        {#each detail.enabled_gateways as gateway}
                            <span>{gateway}</span>
                        {/each}
                    {:else}
                        <p class="state-copy">No gateways are enabled for this agent.</p>
                    {/if}
                </div>
            </article>
        </div>
    {:else}
        <p class="state-copy">No agent detail could be loaded.</p>
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
        font-size: 1.15rem;
    }

    .lead,
    .state-copy,
    .error-copy,
    .detail-card p,
    dd {
        color: var(--mc-text-secondary);
        line-height: 1.55;
    }

    .detail-grid {
        display: grid;
        gap: 0.75rem;
    }

    .detail-card {
        display: grid;
        gap: 0.65rem;
        padding: 1rem 1.1rem;
        border: 1px solid var(--mc-border);
        border-radius: var(--mc-radius-card);
        background: var(--mc-surface);
        box-shadow: 0 10px 18px rgba(30, 36, 48, 0.05);
    }

    .detail-card--summary {
        gap: 0.85rem;
    }

    .summary-row {
        display: flex;
        justify-content: space-between;
        align-items: start;
        gap: 1rem;
    }

    .meta-list {
        display: grid;
        gap: 0.6rem;
    }

    .meta-list div {
        display: grid;
        gap: 0.15rem;
    }

    dt {
        color: var(--mc-text);
        font-size: 0.82rem;
        letter-spacing: 0.08em;
        text-transform: uppercase;
    }

    dd {
        margin: 0;
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

    .binding-cloud {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
    }

    .binding-cloud span {
        width: fit-content;
        padding: 0.25rem 0.55rem;
        border-radius: 999px;
        background: rgba(91, 192, 235, 0.12);
        color: var(--mc-text);
        font-size: 0.83rem;
    }

    .binding-cloud .state-copy {
        width: 100%;
    }

    @media (max-width: 960px) {
        .summary-row {
            flex-direction: column;
        }
    }
</style>
