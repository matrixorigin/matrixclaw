<script lang="ts">
    import { executionPriority, setupSteps } from "$lib/shell";
    import {
        defaultSetupDraft,
        reviewChecklist,
        setupCopy
    } from "$lib/setup/state";
</script>

<section class="setup-shell">
    <aside>
        <p class="section-label">Setup flow</p>
        <h2>First-launch wizard scaffold</h2>
        <ol>
            {#each setupSteps as step, index}
                <li class:index-active={index === 0}>
                    <span>{index + 1}</span>
                    <div>
                        <strong>{step}</strong>
                        <small>
                            {#if step === "Execution"}
                                Docker first, BoxLite second, with visible failure if unavailable.
                            {:else if step === "Review"}
                                Persist config and enter workspace shell.
                            {:else}
                                Capture one stable runtime decision at a time.
                            {/if}
                        </small>
                    </div>
                </li>
            {/each}
        </ol>
    </aside>

    <div class="panel">
        <p class="section-label">Step 1</p>
        <h3>{setupCopy.headline}</h3>
        <p>
            {setupCopy.body}
        </p>

        <div class="form-preview">
            <div>
                <span class="field-label">Provider</span>
                <strong>{defaultSetupDraft.providerName}</strong>
            </div>
            <div>
                <span class="field-label">Model</span>
                <strong>{defaultSetupDraft.model}</strong>
            </div>
            <div>
                <span class="field-label">Workspace</span>
                <strong>{defaultSetupDraft.workspaceName}</strong>
            </div>
            <div>
                <span class="field-label">Root</span>
                <strong>{defaultSetupDraft.workspaceRoot}</strong>
            </div>
            <div>
                <span class="field-label">Execution</span>
                <strong>{defaultSetupDraft.executionMode}</strong>
            </div>
        </div>

        <div class="execution-callout">
            <h4>Execution policy</h4>
            <p>Preferred sandbox order:</p>
            <ul>
                {#each executionPriority as backend, index}
                    <li>{index + 1}. {backend}</li>
                {/each}
            </ul>
        </div>

        <div class="review-callout">
            <h4>Submission contract</h4>
            <ul>
                {#each reviewChecklist as item}
                    <li>{item}</li>
                {/each}
            </ul>
            <p class="endpoint-copy">
                Submit to <code>/api/setup/config</code> and transition to workspace only after
                backend validation succeeds.
            </p>
        </div>
    </div>
</section>

<style>
    .setup-shell {
        display: grid;
        grid-template-columns: minmax(18rem, 23rem) 1fr;
        gap: 1rem;
    }

    aside,
    .panel {
        padding: 1.25rem;
        border-radius: 1.25rem;
        border: 1px solid rgba(148, 163, 184, 0.16);
        background: rgba(15, 23, 42, 0.72);
    }

    .section-label {
        margin: 0 0 0.45rem;
        color: #93c5fd;
        letter-spacing: 0.16em;
        text-transform: uppercase;
        font-size: 0.78rem;
    }

    h2,
    h3,
    h4 {
        margin: 0 0 0.65rem;
    }

    ol,
    ul {
        margin: 0;
        padding-left: 0;
        list-style: none;
    }

    li {
        display: grid;
        grid-template-columns: auto 1fr;
        gap: 0.85rem;
        margin-bottom: 0.85rem;
        padding: 0.85rem;
        border-radius: 1rem;
        background: rgba(30, 41, 59, 0.64);
    }

    li span {
        display: inline-grid;
        place-items: center;
        width: 1.9rem;
        height: 1.9rem;
        border-radius: 999px;
        background: rgba(251, 191, 36, 0.2);
        color: #fde68a;
        font-weight: 700;
    }

    li small,
    p {
        color: #cbd5e1;
        line-height: 1.55;
    }

    .index-active {
        outline: 1px solid rgba(251, 191, 36, 0.35);
    }

    .execution-callout {
        margin-top: 1.2rem;
        padding: 1rem;
        border-radius: 1rem;
        background: rgba(2, 6, 23, 0.45);
    }

    .form-preview {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
        gap: 0.85rem;
        margin-top: 1rem;
    }

    .form-preview div,
    .review-callout {
        padding: 0.95rem;
        border-radius: 1rem;
        background: rgba(2, 6, 23, 0.45);
    }

    .field-label {
        display: block;
        margin-bottom: 0.35rem;
        color: #93c5fd;
        font-size: 0.8rem;
        text-transform: uppercase;
        letter-spacing: 0.12em;
    }

    strong {
        color: #f8fafc;
    }

    .review-callout {
        margin-top: 1rem;
    }

    .endpoint-copy code {
        color: #fde68a;
        font-family:
            "IBM Plex Mono",
            monospace;
    }

    @media (max-width: 820px) {
        .setup-shell {
            grid-template-columns: 1fr;
        }
    }
</style>
