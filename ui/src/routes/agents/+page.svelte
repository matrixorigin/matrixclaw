<script lang="ts">
    import { onMount } from "svelte";

    import { agentRoute, fetchAgents, type AgentSummary } from "$lib/agents";
    import { defaultSelectedAgentSession } from "$lib/agents/session";

    let agents: AgentSummary[] = [];
    let loading = true;
    let pageError = "";

    async function loadAgents() {
        loading = true;
        pageError = "";

        try {
            agents = await fetchAgents();
        } catch (error) {
            pageError = error instanceof Error ? error.message : String(error);
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        void loadAgents();
    });
</script>

<svelte:head>
    <title>Agents | MatrixClaw</title>
</svelte:head>

<section class="route-shell">
    <header class="route-header">
        <p class="section-label">Agents</p>
        <h1>Agents</h1>
        <p class="lead">Available agents and their current capability mix.</p>
        <a class="quick-action" href={agentRoute(defaultSelectedAgentSession.agentName)}>
            Open {defaultSelectedAgentSession.agentName}
        </a>
        <span class="status-pill">Per-agent configuration lives on the detail page.</span>
    </header>

    {#if pageError}
        <p class="error-copy" role="alert">{pageError}</p>
    {/if}

    {#if loading}
        <p class="state-copy">Loading agents...</p>
    {:else if agents.length > 0}
        <div class="agent-list">
            {#each agents as agent}
                <a class="agent-card" href={agentRoute(agent.agent_name)}>
                    <div class="agent-card__copy">
                        <strong>{agent.title}</strong>
                        <p>{agent.crown_job}</p>
                    </div>

                    <div class="agent-card__meta" aria-label={`Bindings for ${agent.title}`}>
                        <span>Skills {agent.enabled_skills.length}</span>
                        <span>MCP {agent.enabled_mcp_servers.length}</span>
                        <span>Gateway {agent.enabled_gateways.length}</span>
                        <span>Bindings {agent.binding_count}</span>
                    </div>
                </a>
            {/each}
        </div>
    {:else}
        <p class="state-copy">No agents were returned by the directory.</p>
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
    .agent-card__copy p {
        color: var(--mc-text-secondary);
        line-height: 1.55;
    }

    .quick-action {
        justify-self: start;
        padding: 0.4rem 0.7rem;
        border: 1px solid var(--mc-border);
        border-radius: 999px;
        background: rgba(91, 192, 235, 0.12);
        color: var(--mc-text);
        font-size: 0.86rem;
    }

    .agent-list {
        display: grid;
        gap: 0.75rem;
    }

    .agent-card {
        display: grid;
        gap: 0.9rem;
        padding: 1rem 1.1rem;
        border: 1px solid var(--mc-border);
        border-radius: var(--mc-radius-card);
        background: var(--mc-surface);
        box-shadow: 0 10px 18px rgba(30, 36, 48, 0.05);
    }

    .agent-card strong {
        color: var(--mc-text);
        font-size: 1.02rem;
    }

    .agent-card__meta {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
    }

    .agent-card__meta span {
        padding: 0.3rem 0.55rem;
        border-radius: 999px;
        background: rgba(91, 192, 235, 0.12);
        color: var(--mc-text);
        font-size: 0.84rem;
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
</style>
