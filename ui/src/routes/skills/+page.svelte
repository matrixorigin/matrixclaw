<script lang="ts">
    import {
        enabledSkillState,
        installedSkills,
        isSkillEnabled,
        toggleSkill,
        type EnabledSkillState,
        type InstalledSkill
    } from "$lib/skills";

    let selectedSkill: InstalledSkill = installedSkills[0];
    let skillState: EnabledSkillState = enabledSkillState;

    function selectSkill(skill: InstalledSkill) {
        selectedSkill = skill;
    }

    function flipSelectedSkill() {
        skillState = toggleSkill(selectedSkill.name, skillState);
    }
</script>

<section class="skills-shell">
    <aside class="inventory">
        <p class="section-label">Installed skills</p>
        <h2>Global inventory</h2>
        <p class="lead">
            Installed skill packages stay immutable. Agent-local enablement is tracked
            separately so operators can reuse the same imports safely.
        </p>

        <div class="skill-list">
            {#each installedSkills as skill}
                <button
                    type="button"
                    class:selected={skill.name === selectedSkill.name}
                    on:click={() => selectSkill(skill)}
                >
                    <strong>{skill.name}</strong>
                    <span>{skill.supportTier}</span>
                    <small>{skill.source}</small>
                </button>
            {/each}
        </div>
    </aside>

    <article class="detail">
        <p class="section-label">Selected skill</p>
        <div class="title-row">
            <div>
                <h2>{selectedSkill.name}</h2>
                <p>{selectedSkill.description}</p>
            </div>
            <span
                class:enabled={isSkillEnabled(selectedSkill.name, skillState)}
                class="status-pill"
            >
                {#if isSkillEnabled(selectedSkill.name, skillState)}
                    Enabled for {skillState.agentName}
                {:else}
                    Installed only
                {/if}
            </span>
        </div>

        <dl class="meta-grid">
            <div>
                <dt>Version</dt>
                <dd>{selectedSkill.version}</dd>
            </div>
            <div>
                <dt>Compatibility</dt>
                <dd>{selectedSkill.supportTier}</dd>
            </div>
            <div>
                <dt>Agent-local state</dt>
                <dd>{skillState.agentName}</dd>
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

        <button type="button" class="toggle" on:click={flipSelectedSkill}>
            {#if isSkillEnabled(selectedSkill.name, skillState)}
                Disable for {skillState.agentName}
            {:else}
                Enable for {skillState.agentName}
            {/if}
        </button>
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

    @media (max-width: 860px) {
        .skills-shell {
            grid-template-columns: 1fr;
        }
    }
</style>
