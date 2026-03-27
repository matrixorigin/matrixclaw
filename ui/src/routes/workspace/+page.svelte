<script lang="ts">
    const fileEntries = [
        "agents/default/SOUL.md",
        "agents/default/MEMORY.md",
        "skills/research/SKILL.md",
        "sessions/default/latest.jsonl"
    ];

    const transcriptEntries = [
        {
            role: "assistant",
            text: "Workspace shell scaffold is ready for streamed transcript work."
        },
        {
            role: "tool",
            text: "Execution badges will expose local, docker, and boxlite provenance."
        }
    ];

    const runFacts = [
        "Active run: idle",
        "Queue state: empty",
        "Execution backend: docker",
        "Fallback policy: require sandbox"
    ];
</script>

<section class="workspace-shell">
    <aside class="left-rail">
        <p class="section-label">Workspace</p>
        <h2>Files and navigation</h2>
        <ul>
            {#each fileEntries as entry}
                <li>
                    <span>{entry}</span>
                    <button type="button">Reference</button>
                </li>
            {/each}
        </ul>
    </aside>

    <div class="main-column">
        <div class="transcript">
            <p class="section-label">Transcript</p>
            {#each transcriptEntries as item}
                <article data-role={item.role}>
                    <strong>{item.role}</strong>
                    <p>{item.text}</p>
                </article>
            {/each}
        </div>

        <form class="composer">
            <label for="prompt">Composer</label>
            <textarea
                id="prompt"
                rows="4"
                placeholder="Ask the agent, attach references, or queue a steering update."
            ></textarea>
            <div class="composer-actions">
                <span>Reference chips land here</span>
                <button type="button">Send</button>
            </div>
        </form>
    </div>

    <aside class="right-rail">
        <p class="section-label">Run state</p>
        <h2>Queue and execution detail</h2>
        <ul>
            {#each runFacts as fact}
                <li>{fact}</li>
            {/each}
        </ul>
    </aside>
</section>

<style>
    .workspace-shell {
        display: grid;
        grid-template-columns: minmax(16rem, 20rem) minmax(0, 1fr) minmax(16rem, 20rem);
        gap: 1rem;
    }

    .left-rail,
    .main-column,
    .right-rail,
    .transcript,
    .composer {
        border-radius: 1.25rem;
        border: 1px solid rgba(148, 163, 184, 0.16);
        background: rgba(15, 23, 42, 0.72);
    }

    .left-rail,
    .right-rail,
    .transcript,
    .composer {
        padding: 1.1rem;
    }

    .section-label {
        margin: 0 0 0.45rem;
        color: #86efac;
        letter-spacing: 0.16em;
        text-transform: uppercase;
        font-size: 0.78rem;
    }

    h2 {
        margin: 0 0 0.85rem;
    }

    ul {
        margin: 0;
        padding: 0;
        list-style: none;
    }

    .left-rail li {
        display: grid;
        gap: 0.65rem;
        margin-bottom: 0.75rem;
        padding: 0.85rem;
        border-radius: 0.95rem;
        background: rgba(30, 41, 59, 0.66);
    }

    button {
        width: fit-content;
        padding: 0.6rem 0.9rem;
        border: 0;
        border-radius: 999px;
        background: #fbbf24;
        color: #111827;
        font-weight: 700;
        cursor: pointer;
    }

    .main-column {
        display: grid;
        gap: 1rem;
        background: transparent;
        border: 0;
    }

    .transcript {
        min-height: 21rem;
    }

    article {
        margin-top: 0.85rem;
        padding: 0.95rem;
        border-radius: 1rem;
        background: rgba(2, 6, 23, 0.45);
    }

    article[data-role="tool"] {
        border-left: 3px solid #f59e0b;
    }

    article p,
    .right-rail li,
    .left-rail span,
    .composer-actions span {
        color: #cbd5e1;
        line-height: 1.55;
    }

    .composer {
        display: grid;
        gap: 0.75rem;
    }

    textarea {
        width: 100%;
        resize: vertical;
        min-height: 8rem;
        padding: 1rem;
        border: 1px solid rgba(148, 163, 184, 0.2);
        border-radius: 1rem;
        background: rgba(2, 6, 23, 0.45);
        color: inherit;
        font: inherit;
    }

    .composer-actions {
        display: flex;
        flex-wrap: wrap;
        gap: 0.75rem;
        justify-content: space-between;
        align-items: center;
    }

    .right-rail li {
        margin-bottom: 0.7rem;
        padding: 0.85rem;
        border-radius: 0.95rem;
        background: rgba(30, 41, 59, 0.66);
    }

    @media (max-width: 980px) {
        .workspace-shell {
            grid-template-columns: 1fr;
        }
    }
</style>
