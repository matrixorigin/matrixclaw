<script lang="ts">
    import { errorMessage, fetchJson } from "$lib/http";
    import { onMount } from "svelte";

    type InstalledSkillRecord = {
        name: string;
        source_root: string;
        installed_root: string;
        manifest_path: string;
        provenance_path: string;
    };

    type EnabledSkillsRecord = {
        agent_name: string;
        enabled: string[];
    };

    type SkillsInventory = {
        installed: InstalledSkillRecord[];
        enabled: EnabledSkillsRecord[];
    };

    const agentName = "default";

    let inventory: InstalledSkillRecord[] = [];
    let enabledState: EnabledSkillsRecord = {
        agent_name: agentName,
        enabled: []
    };
    let selectedSkill: InstalledSkillRecord | null = null;
    let pageError = "";
    let busy = false;

    onMount(async () => {
        await loadInventory();
    });

    function isSkillEnabled(skillName: string): boolean {
        return enabledState.enabled.includes(skillName);
    }

    function supportTier(skill: InstalledSkillRecord): "native" | "shimmed" {
        return skill.name.includes("bridge") ? "shimmed" : "native";
    }

    function descriptionFor(skill: InstalledSkillRecord): string {
        return supportTier(skill) === "native"
            ? "Imported as a native MatrixClaw skill package."
            : "Imported through a shimmed compatibility boundary.";
    }

    async function loadInventory() {
        pageError = "";

        try {
            const response = await fetchJson<SkillsInventory>(`/api/skills?agent=${agentName}`);
            inventory = response.installed;
            enabledState = response.enabled[0] ?? {
                agent_name: agentName,
                enabled: []
            };
            selectedSkill = inventory[0] ?? null;
        } catch (error) {
            pageError = errorMessage(error);
        }
    }

    async function toggleSelectedSkill() {
        if (!selectedSkill) {
            return;
        }

        busy = true;
        pageError = "";

        try {
            enabledState = await fetchJson<EnabledSkillsRecord>("/api/skills/toggle", {
                method: "POST",
                body: JSON.stringify({
                    agent_name: enabledState.agent_name,
                    skill_name: selectedSkill.name,
                    enabled: !isSkillEnabled(selectedSkill.name)
                })
            });
        } catch (error) {
            pageError = errorMessage(error);
        } finally {
            busy = false;
        }
    }
</script>

<section class="skills-shell">
    <aside class="inventory">
        <p class="section-label">Installed skills</p>
        <h2>Global inventory</h2>
        <p class="lead">
            Installed skill packages stay immutable. Agent-local enablement is tracked separately
            so operators can reuse the same imports safely.
        </p>

        {#if pageError}
            <p class="error-copy">{pageError}</p>
        {/if}

        <div class="skill-list">
            {#each inventory as skill}
                <button
                    type="button"
                    class:selected={skill.name === selectedSkill?.name}
                    on:click={() => (selectedSkill = skill)}
                >
                    <strong>{skill.name}</strong>
                    <span>{supportTier(skill)}</span>
                    <small>{skill.source_root}</small>
                </button>
            {/each}
        </div>
    </aside>

    <article class="detail">
        {#if selectedSkill}
            <p class="section-label">Selected skill</p>
            <div class="title-row">
                <div>
                    <h2>{selectedSkill.name}</h2>
                    <p>{descriptionFor(selectedSkill)}</p>
                </div>
                <span class:enabled={isSkillEnabled(selectedSkill.name)} class="status-pill">
                    {#if isSkillEnabled(selectedSkill.name)}
                        Enabled for {enabledState.agent_name}
                    {:else}
                        Installed only
                    {/if}
                </span>
            </div>

            <dl class="meta-grid">
                <div>
                    <dt>Compatibility</dt>
                    <dd>{supportTier(selectedSkill)}</dd>
                </div>
                <div>
                    <dt>Agent-local state</dt>
                    <dd>{enabledState.agent_name}</dd>
                </div>
                <div>
                    <dt>Manifest</dt>
                    <dd>{selectedSkill.manifest_path}</dd>
                </div>
                <div>
                    <dt>Provenance</dt>
                    <dd>{selectedSkill.provenance_path}</dd>
                </div>
                <div>
                    <dt>Installed root</dt>
                    <dd>{selectedSkill.installed_root}</dd>
                </div>
                <div>
                    <dt>Mutation boundary</dt>
                    <dd>`enabled-skills.json` only</dd>
                </div>
            </dl>

            <div class="callout">
                <h3>Enablement safety</h3>
                <p>
                    Toggling enablement updates only agent metadata. Imported packages and upstream
                    source files are left untouched.
                </p>
            </div>

            <button type="button" class="toggle" on:click={toggleSelectedSkill} disabled={busy}>
                {#if isSkillEnabled(selectedSkill.name)}
                    Disable for {enabledState.agent_name}
                {:else}
                    Enable for {enabledState.agent_name}
                {/if}
            </button>
        {:else}
            <p class="lead">No installed skills were found for this agent yet.</p>
        {/if}
    </article>
</section>

<style>
    .skills-shell {
        display: grid;
        grid-template-columns: minmax(18rem, 24rem) minmax(0, 1fr);
        gap: 1rem;
    }

    .inventory,
    .detail {
        padding: 1.2rem;
        border-radius: 1.25rem;
        border: 1px solid rgba(148, 163, 184, 0.16);
        background: rgba(15, 23, 42, 0.72);
    }

    .section-label {
        margin: 0 0 0.45rem;
        color: #c4b5fd;
        letter-spacing: 0.16em;
        text-transform: uppercase;
        font-size: 0.78rem;
    }

    .lead,
    p,
    dd,
    small {
        color: #cbd5e1;
        line-height: 1.55;
    }

    .error-copy {
        color: #fecaca;
    }

    h2,
    h3 {
        margin: 0 0 0.7rem;
    }

    .skill-list {
        display: grid;
        gap: 0.75rem;
        margin-top: 1rem;
    }

    .skill-list button {
        display: grid;
        gap: 0.25rem;
        padding: 0.95rem;
        text-align: left;
        border-radius: 1rem;
        border: 1px solid rgba(148, 163, 184, 0.16);
        background: rgba(30, 41, 59, 0.66);
        color: inherit;
        cursor: pointer;
    }

    .skill-list button.selected {
        border-color: rgba(196, 181, 253, 0.55);
        background: rgba(49, 46, 129, 0.42);
    }

    .skill-list span {
        width: fit-content;
        padding: 0.15rem 0.55rem;
        border-radius: 999px;
        background: rgba(196, 181, 253, 0.14);
        color: #ddd6fe;
        font-size: 0.82rem;
        text-transform: capitalize;
    }

    .title-row {
        display: flex;
        gap: 1rem;
        justify-content: space-between;
        align-items: flex-start;
    }

    .status-pill {
        padding: 0.45rem 0.75rem;
        border-radius: 999px;
        background: rgba(148, 163, 184, 0.18);
        color: #cbd5e1;
        white-space: nowrap;
    }

    .status-pill.enabled {
        background: rgba(74, 222, 128, 0.18);
        color: #bbf7d0;
    }

    .meta-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
        gap: 0.85rem;
        margin: 1.2rem 0;
    }

    dt {
        margin-bottom: 0.3rem;
        color: #94a3b8;
        font-size: 0.82rem;
        text-transform: uppercase;
        letter-spacing: 0.08em;
    }

    dd {
        margin: 0;
        overflow-wrap: anywhere;
    }

    .callout {
        margin-bottom: 1rem;
        padding: 1rem;
        border-radius: 1rem;
        background: rgba(2, 6, 23, 0.45);
    }

    .toggle {
        padding: 0.75rem 1rem;
        border: 0;
        border-radius: 999px;
        background: #ddd6fe;
        color: #1e1b4b;
        font-weight: 700;
        cursor: pointer;
    }

    .toggle:disabled {
        opacity: 0.55;
        cursor: not-allowed;
    }

    @media (max-width: 860px) {
        .skills-shell {
            grid-template-columns: 1fr;
        }
    }
</style>
