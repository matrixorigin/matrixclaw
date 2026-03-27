<script lang="ts">
    import { goto } from "$app/navigation";
    import { errorMessage, fetchJson } from "$lib/http";
    import { executionPriority, setupSteps } from "$lib/shell";
    import { sandboxFailureMessage, visibleExecutionBackends } from "$lib/execution";
    import { defaultSetupDraft, reviewChecklist, setupCopy } from "$lib/setup/state";
    import { onMount } from "svelte";

    type HealthSnapshot = {
        mode: string;
        baseUrl: string;
        configReady: boolean;
    };

    type SetupValidationResponse = {
        accepted: boolean;
        configWritten: boolean;
        next?: string | null;
        error?: string | null;
    };

    const draft = { ...defaultSetupDraft };
    let health: HealthSnapshot | null = null;
    let submitError = "";
    let submitMessage = "";
    let submitting = false;

    onMount(async () => {
        try {
            health = await fetchJson<HealthSnapshot>("/healthz");
        } catch (error) {
            submitError = errorMessage(error);
        }
    });

    async function submitSetup() {
        submitError = "";
        submitMessage = "";
        submitting = true;

        try {
            const response = await fetchJson<SetupValidationResponse>("/api/setup/config", {
                method: "POST",
                body: JSON.stringify({
                    provider: {
                        provider_name: draft.providerName,
                        model: draft.model
                    },
                    workspace: {
                        name: draft.workspaceName,
                        root: draft.workspaceRoot
                    },
                    auth: {
                        token: draft.authToken
                    },
                    execution: {
                        mode: draft.executionMode === "sandboxed" ? "Sandboxed" : "Local",
                        backend:
                            draft.executionMode === "sandboxed"
                                ? {
                                      kind: "Sandbox",
                                      label: "sandbox",
                                      requires_docker: false
                                  }
                                : {
                                      kind: "LocalCommand",
                                      label: "local-command",
                                      requires_docker: false
                                  }
                    }
                })
            });

            if (response.configWritten) {
                submitMessage = "Configuration saved. Opening workspace...";
                await goto("/workspace");
            } else {
                submitError = response.error ?? "setup submission was rejected";
            }
        } catch (error) {
            submitError = errorMessage(error);
        } finally {
            submitting = false;
        }
    }
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
                <input bind:value={draft.providerName} />
            </div>
            <div>
                <span class="field-label">Model</span>
                <input bind:value={draft.model} />
            </div>
            <div>
                <span class="field-label">Workspace</span>
                <input bind:value={draft.workspaceName} />
            </div>
            <div>
                <span class="field-label">Root</span>
                <input bind:value={draft.workspaceRoot} />
            </div>
            <div>
                <span class="field-label">Auth token</span>
                <input bind:value={draft.authToken} placeholder="sk-..." />
            </div>
            <div>
                <span class="field-label">Execution</span>
                <select bind:value={draft.executionMode}>
                    <option value="local">local</option>
                    <option value="sandboxed">sandboxed</option>
                </select>
            </div>
        </div>

        <div class="execution-callout">
            <h4>Execution policy</h4>
            <p class="support-copy">Visible backends:</p>
            <div class="backend-chips">
                {#each visibleExecutionBackends as backend}
                    <span>{backend}</span>
                {/each}
            </div>
            <p>Preferred sandbox order:</p>
            <ul>
                {#each executionPriority as backend, index}
                    <li>{index + 1}. {backend}</li>
                {/each}
            </ul>
            <p class="failure-copy">{sandboxFailureMessage}</p>
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
            {#if health}
                <p class="endpoint-copy">Health mode: <code>{health.mode}</code></p>
            {/if}
            {#if submitMessage}
                <p class="success-copy">{submitMessage}</p>
            {/if}
            {#if submitError}
                <p class="error-copy">{submitError}</p>
            {/if}
            <button type="button" class="submit-button" on:click={submitSetup} disabled={submitting}>
                {#if submitting}
                    Saving...
                {:else}
                    Save and continue
                {/if}
            </button>
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

    .support-copy {
        margin-bottom: 0.55rem;
    }

    .backend-chips {
        display: flex;
        flex-wrap: wrap;
        gap: 0.6rem;
        margin-bottom: 0.85rem;
    }

    .backend-chips span {
        display: inline-flex;
        align-items: center;
        padding: 0.45rem 0.75rem;
        border-radius: 999px;
        background: rgba(59, 130, 246, 0.14);
        color: #dbeafe;
        font-size: 0.85rem;
        text-transform: uppercase;
        letter-spacing: 0.08em;
    }

    .failure-copy {
        margin-top: 0.85rem;
        color: #fde68a;
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

    input,
    select {
        width: 100%;
        padding: 0.7rem 0.8rem;
        border: 1px solid rgba(148, 163, 184, 0.2);
        border-radius: 0.85rem;
        background: rgba(15, 23, 42, 0.64);
        color: inherit;
        font: inherit;
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

    .submit-button {
        margin-top: 0.8rem;
        padding: 0.75rem 1rem;
        border: 0;
        border-radius: 999px;
        background: #fbbf24;
        color: #111827;
        font-weight: 700;
        cursor: pointer;
    }

    .submit-button:disabled {
        opacity: 0.65;
        cursor: progress;
    }

    .success-copy {
        color: #86efac;
    }

    .error-copy {
        color: #fca5a5;
    }

    @media (max-width: 820px) {
        .setup-shell {
            grid-template-columns: 1fr;
        }
    }
</style>
