<script lang="ts">
    import { browser } from "$app/environment";
    import { goto } from "$app/navigation";
    import { errorMessage, fetchJson } from "$lib/http";
    import { sandboxFailureMessage, visibleExecutionBackends } from "$lib/execution";
    import {
        defaultSetupDraft,
        reviewChecklist,
        setupCopy,
        setupDraftStorageKey,
        setupStepStorageKey
    } from "$lib/setup/state";
    import {
        buildSetupPayload,
        completedStepCount,
        maskedToken,
        setupFlowSteps,
        stepIsComplete,
        stepIssues,
        type SetupFlowDraft
    } from "$lib/setup/flow";
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

    let draft: SetupFlowDraft = { ...defaultSetupDraft };
    let health: HealthSnapshot | null = null;
    let submitError = "";
    let submitMessage = "";
    let submitting = false;
    let currentStep = 0;
    let restoredFromStorage = false;

    if (browser) {
        restoreDraft();
        restoredFromStorage = true;
    }

    onMount(async () => {
        try {
            health = await fetchJson<HealthSnapshot>("/healthz");
        } catch (error) {
            submitError = errorMessage(error);
        }
    });

    function moveToStep(index: number) {
        currentStep = index;
        submitError = "";
        submitMessage = "";
        if (restoredFromStorage) {
            persistDraft();
        }
    }

    function restoreDraft() {
        if (typeof localStorage === "undefined") {
            return;
        }

        const storedDraft = localStorage.getItem(setupDraftStorageKey);
        if (storedDraft) {
            try {
                draft = {
                    ...draft,
                    ...(JSON.parse(storedDraft) as Partial<SetupFlowDraft>)
                };
            } catch {
                localStorage.removeItem(setupDraftStorageKey);
            }
        }

        const storedStep = localStorage.getItem(setupStepStorageKey);
        if (storedStep) {
            const parsed = Number.parseInt(storedStep, 10);
            if (Number.isFinite(parsed)) {
                currentStep = Math.max(0, Math.min(parsed, setupFlowSteps.length - 1));
            }
        }
    }

    function persistDraft() {
        if (typeof localStorage === "undefined") {
            return;
        }

        localStorage.setItem(setupDraftStorageKey, JSON.stringify(draft));
        localStorage.setItem(setupStepStorageKey, String(currentStep));
    }

    function persistDraftChange() {
        if (restoredFromStorage) {
            persistDraft();
        }
    }

    function nextStep() {
        if (activeStepIssues.length > 0) {
            return;
        }

        if (currentStep < setupFlowSteps.length - 1) {
            moveToStep(currentStep + 1);
        }
    }

    function previousStep() {
        if (currentStep > 0) {
            moveToStep(currentStep - 1);
        }
    }

    $: activeStep = setupFlowSteps[currentStep];
    $: activeStepIssues = stepIssues(draft, activeStep.id);
    $: completedSteps = completedStepCount(draft);
    $: progressPercent = Math.round((completedSteps / (setupFlowSteps.length - 1)) * 100);

    async function submitSetup() {
        if (stepIssues(draft, "review").length > 0) {
            currentStep = setupFlowSteps.length - 1;
            submitError = "Resolve the remaining setup issues before saving.";
            return;
        }

        submitError = "";
        submitMessage = "";
        submitting = true;

        try {
            const response = await fetchJson<SetupValidationResponse>("/api/setup/config", {
                method: "POST",
                body: JSON.stringify(buildSetupPayload(draft))
            });

            if (response.configWritten) {
                if (typeof localStorage !== "undefined") {
                    localStorage.removeItem(setupDraftStorageKey);
                    localStorage.removeItem(setupStepStorageKey);
                }
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
    <aside class="stepper-panel">
        <div class="stepper-header">
            <p class="section-label">Onboarding flow</p>
            <h2>Desktop first-launch setup</h2>
            <p>
                Replace the old scaffold with a real stepper: choose the runtime contract, validate
                each decision, then persist once.
            </p>
        </div>

        <div class="progress-panel">
            <div class="progress-copy">
                <span>Completed</span>
                <strong>{completedSteps} / {setupFlowSteps.length - 1}</strong>
            </div>
            <div class="progress-bar" aria-hidden="true">
                <span style={`width: ${progressPercent}%`}></span>
            </div>
        </div>

        <ol>
            {#each setupFlowSteps as step, index}
                <li
                    class:active={index === currentStep}
                    class:complete={step.id !== "review" && stepIsComplete(draft, step.id)}
                >
                    <button type="button" on:click={() => moveToStep(index)}>
                        <span>{index + 1}</span>
                        <div>
                            <strong>{step.title}</strong>
                            <small>{step.detail}</small>
                        </div>
                    </button>
                </li>
            {/each}
        </ol>

        <div class="system-card">
            <p class="section-label">Runtime probe</p>
            <strong>{health ? health.mode : "Awaiting loopback runtime"}</strong>
            <p>
                {#if health}
                    Health endpoint: <code>{health.baseUrl}</code>
                {:else}
                    Setup can still be prepared while the loopback runtime reconnects.
                {/if}
            </p>
        </div>

        <div class="system-card launch-card">
            <p class="section-label">Launch contract</p>
            <dl>
                <div>
                    <dt>Provider</dt>
                    <dd>{draft.providerName}</dd>
                </div>
                <div>
                    <dt>Model</dt>
                    <dd>{draft.model}</dd>
                </div>
                <div>
                    <dt>Workspace</dt>
                    <dd>{draft.workspaceName}</dd>
                </div>
            </dl>
        </div>
    </aside>

    <div class="panel">
        <div class="hero-panel">
            <div>
                <p class="section-label">{activeStep.eyebrow}</p>
                <h3>{activeStep.title}</h3>
                <p>{activeStep.description}</p>
            </div>
            <div class="hero-note">
                <strong>{setupCopy.headline}</strong>
                <p>{setupCopy.body}</p>
            </div>
        </div>

        {#if activeStep.id === "provider"}
            <div class="form-grid">
                <label class="field-card">
                    <span class="field-label">Provider name</span>
                    <input
                        bind:value={draft.providerName}
                        placeholder="openai-compatible"
                        on:input={persistDraftChange}
                    />
                    <small>Names the runtime bridge persisted by setup.</small>
                </label>

                <label class="field-card">
                    <span class="field-label">Default model</span>
                    <input bind:value={draft.model} placeholder="gpt-5.4" on:input={persistDraftChange} />
                    <small>This becomes the initial model surfaced by the app shell.</small>
                </label>
            </div>

            <div class="support-panel">
                <p class="support-label">Provider contract</p>
                <p>
                    Keep provider identity explicit. The desktop shell should never hide which model
                    the first workspace session will use.
                </p>
            </div>
        {:else if activeStep.id === "workspace"}
            <div class="form-grid">
                <label class="field-card">
                    <span class="field-label">Workspace name</span>
                    <input bind:value={draft.workspaceName} placeholder="default" on:input={persistDraftChange} />
                    <small>Displayed throughout the app shell as the active project label.</small>
                </label>

                <label class="field-card wide">
                    <span class="field-label">Workspace root</span>
                    <input
                        bind:value={draft.workspaceRoot}
                        placeholder="~/workspace"
                        on:input={persistDraftChange}
                    />
                    <small>Used by the runtime when file references and workspace APIs resolve.</small>
                </label>
            </div>

            <div class="support-panel">
                <p class="support-label">Why this step matters</p>
                <p>
                    The setup flow should bind the shell to a concrete root before the user lands in
                    the workspace route.
                </p>
            </div>
        {:else if activeStep.id === "auth"}
            <div class="form-grid">
                <label class="field-card wide">
                    <span class="field-label">Auth token</span>
                    <input bind:value={draft.authToken} placeholder="sk-..." on:input={persistDraftChange} />
                    <small>The setup flow keeps auth explicit so provider failures surface early.</small>
                </label>
            </div>

            <div class="support-panel">
                <p class="support-label">Stored preview</p>
                <p>{maskedToken(draft.authToken)}</p>
            </div>
        {:else if activeStep.id === "execution"}
            <div class="form-grid">
                <label class="field-card">
                    <span class="field-label">Execution mode</span>
                    <select bind:value={draft.executionMode} on:change={persistDraftChange}>
                        <option value="local">local</option>
                        <option value="sandboxed">sandboxed</option>
                    </select>
                    <small>Choose the default lane the runtime exposes after setup.</small>
                </label>

                <article class="field-card">
                    <span class="field-label">Visible backends</span>
                    <div class="backend-chips">
                        {#each visibleExecutionBackends as backend}
                            <span>{backend}</span>
                        {/each}
                    </div>
                    <small>Failure remains explicit instead of silently changing behavior.</small>
                </article>
            </div>

            <div class="support-panel">
                <p class="support-label">Fallback posture</p>
                <p>{sandboxFailureMessage}</p>
            </div>
        {:else}
            <div class="review-grid">
                <article class="summary-card">
                    <p class="field-label">Provider</p>
                    <strong>{draft.providerName}</strong>
                    <p>{draft.model}</p>
                </article>

                <article class="summary-card">
                    <p class="field-label">Workspace</p>
                    <strong>{draft.workspaceName}</strong>
                    <p>{draft.workspaceRoot}</p>
                </article>

                <article class="summary-card">
                    <p class="field-label">Access</p>
                    <strong>{maskedToken(draft.authToken)}</strong>
                    <p>Persisted through the setup submission contract.</p>
                </article>

                <article class="summary-card">
                    <p class="field-label">Execution</p>
                    <strong>{draft.executionMode}</strong>
                    <p>Visible backends: {visibleExecutionBackends.join(", ")}</p>
                </article>
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
        {/if}

        <div class="issue-strip" class:has-issues={activeStepIssues.length > 0}>
            <div>
                <p class="support-label">Validation</p>
                {#if activeStepIssues.length === 0}
                    <strong>Ready for the next step.</strong>
                {:else}
                    <strong>{activeStepIssues.length} issue(s) to resolve.</strong>
                {/if}
            </div>

            {#if activeStepIssues.length > 0}
                <ul class="issue-list">
                    {#each activeStepIssues as issue}
                        <li>{issue}</li>
                    {/each}
                </ul>
            {/if}
        </div>

        <div class="footer-actions">
            <button type="button" class="ghost-button" on:click={previousStep} disabled={currentStep === 0}>
                Back
            </button>

            <div class="footer-status">
                {#if submitMessage}
                    <p class="success-copy">{submitMessage}</p>
                {/if}
                {#if submitError}
                    <p class="error-copy">{submitError}</p>
                {/if}
            </div>

            {#if activeStep.id === "review"}
                <button
                    type="button"
                    class="submit-button"
                    on:click={submitSetup}
                    disabled={submitting}
                >
                    {#if submitting}
                        Saving...
                    {:else}
                        Save and enter workspace
                    {/if}
                </button>
            {:else}
                <button
                    type="button"
                    class="submit-button"
                    on:click={nextStep}
                    disabled={activeStepIssues.length > 0}
                >
                    Continue
                </button>
            {/if}
        </div>
    </div>
</section>

<style>
    .setup-shell {
        display: grid;
        grid-template-columns: minmax(18rem, 23rem) 1fr;
        gap: 1rem;
        align-items: start;
    }

    .stepper-panel,
    .panel {
        border-radius: 1.4rem;
        border: 1px solid rgba(148, 163, 184, 0.16);
        background:
            linear-gradient(180deg, rgba(255, 255, 255, 0.02), transparent 20%),
            rgba(10, 18, 33, 0.82);
    }

    .stepper-panel {
        display: grid;
        gap: 1rem;
        padding: 1.2rem;
    }

    .panel {
        display: grid;
        gap: 1rem;
        padding: 1.25rem;
    }

    .section-label,
    .field-label,
    .support-label {
        margin: 0;
        color: #7dd3fc;
        letter-spacing: 0.14em;
        text-transform: uppercase;
        font-size: 0.78rem;
    }

    .stepper-header h2,
    .hero-panel h3,
    .review-callout h4 {
        margin: 0 0 0.6rem;
    }

    .stepper-header p,
    .hero-panel p,
    .support-panel p,
    .summary-card p,
    .system-card p,
    .issue-list li,
    small {
        color: #b5c4d8;
        line-height: 1.6;
    }

    .progress-panel,
    .system-card,
    .hero-note,
    .support-panel,
    .issue-strip,
    .review-callout,
    .summary-card,
    .field-card {
        padding: 1rem;
        border-radius: 1.1rem;
        border: 1px solid rgba(148, 163, 184, 0.12);
        background: rgba(6, 13, 24, 0.58);
    }

    .progress-copy {
        display: flex;
        justify-content: space-between;
        gap: 1rem;
        align-items: center;
        margin-bottom: 0.7rem;
        color: #b5c4d8;
    }

    .progress-copy strong,
    .system-card strong,
    .summary-card strong,
    .issue-strip strong {
        color: #eef4ff;
    }

    .progress-bar {
        height: 0.55rem;
        border-radius: 999px;
        background: rgba(15, 23, 42, 0.9);
        overflow: hidden;
    }

    .progress-bar span {
        display: block;
        height: 100%;
        border-radius: inherit;
        background: linear-gradient(90deg, #7dd3fc, #f59e0b);
        transition: width 180ms ease;
    }

    ol,
    ul {
        margin: 0;
        padding: 0;
        list-style: none;
    }

    .launch-card dl {
        display: grid;
        gap: 0.8rem;
        margin: 0;
    }

    .launch-card div {
        display: grid;
        gap: 0.15rem;
    }

    .launch-card dt {
        color: #7dd3fc;
        font-size: 0.74rem;
        letter-spacing: 0.08em;
        text-transform: uppercase;
    }

    .launch-card dd {
        margin: 0;
        color: #eef4ff;
        font-weight: 600;
    }

    ol {
        display: grid;
        gap: 0.75rem;
    }

    li button {
        width: 100%;
        display: grid;
        grid-template-columns: auto 1fr;
        gap: 0.85rem;
        align-items: start;
        padding: 0.9rem;
        border: 1px solid rgba(148, 163, 184, 0.14);
        border-radius: 1rem;
        background: rgba(8, 15, 28, 0.74);
        color: inherit;
        text-align: left;
        cursor: pointer;
        transition:
            transform 160ms ease,
            border-color 160ms ease,
            background 160ms ease;
    }

    li.active button,
    li button:hover {
        transform: translateX(0.12rem);
        border-color: rgba(125, 211, 252, 0.34);
        background: rgba(17, 30, 51, 0.9);
    }

    li.complete button {
        border-color: rgba(52, 211, 153, 0.24);
    }

    li span {
        display: inline-grid;
        place-items: center;
        width: 2rem;
        height: 2rem;
        border-radius: 999px;
        background: rgba(125, 211, 252, 0.16);
        color: #eef4ff;
        font-weight: 700;
    }

    li.complete span {
        background: rgba(52, 211, 153, 0.2);
    }

    .hero-panel {
        display: grid;
        grid-template-columns: minmax(0, 1.5fr) minmax(16rem, 0.9fr);
        gap: 1rem;
    }

    .form-grid,
    .review-grid {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 0.9rem;
    }

    .field-card {
        display: grid;
        gap: 0.55rem;
    }

    .field-card.wide {
        grid-column: 1 / -1;
    }

    input,
    select {
        width: 100%;
        padding: 0.78rem 0.85rem;
        border: 1px solid rgba(148, 163, 184, 0.22);
        border-radius: 0.9rem;
        background: rgba(15, 23, 42, 0.72);
        color: inherit;
        font: inherit;
    }

    .backend-chips {
        display: flex;
        flex-wrap: wrap;
        gap: 0.55rem;
    }

    .backend-chips span {
        display: inline-flex;
        align-items: center;
        padding: 0.42rem 0.7rem;
        border-radius: 999px;
        background: rgba(125, 211, 252, 0.12);
        color: #d8f1ff;
        font-size: 0.82rem;
        letter-spacing: 0.08em;
        text-transform: uppercase;
    }

    .support-panel {
        background:
            radial-gradient(circle at top right, rgba(125, 211, 252, 0.08), transparent 32%),
            rgba(7, 14, 26, 0.72);
    }

    .review-grid {
        grid-template-columns: repeat(auto-fit, minmax(13rem, 1fr));
    }

    .summary-card strong {
        margin: 0.3rem 0;
        font-size: 1rem;
    }

    .issue-strip {
        display: grid;
        gap: 0.8rem;
    }

    .issue-strip.has-issues {
        border-color: rgba(251, 113, 133, 0.24);
    }

    .issue-list {
        display: grid;
        gap: 0.45rem;
    }

    .issue-list li {
        padding-left: 1.1rem;
        position: relative;
    }

    .issue-list li::before {
        content: "";
        position: absolute;
        left: 0;
        top: 0.7rem;
        width: 0.45rem;
        height: 0.45rem;
        border-radius: 999px;
        background: #fb7185;
    }

    .footer-actions {
        display: grid;
        grid-template-columns: auto 1fr auto;
        gap: 1rem;
        align-items: center;
    }

    .footer-status {
        min-height: 1.5rem;
    }

    .ghost-button,
    .submit-button {
        padding: 0.82rem 1.1rem;
        border-radius: 999px;
        font-weight: 700;
        cursor: pointer;
        font: inherit;
    }

    .ghost-button {
        border: 1px solid rgba(148, 163, 184, 0.18);
        background: rgba(255, 255, 255, 0.03);
        color: #eef4ff;
    }

    .submit-button {
        border: 0;
        background: linear-gradient(90deg, #7dd3fc, #f59e0b);
        color: #08111f;
    }

    .ghost-button:disabled,
    .submit-button:disabled {
        opacity: 0.58;
        cursor: not-allowed;
    }

    .endpoint-copy code,
    .system-card code {
        color: #fcd34d;
        font-family:
            "IBM Plex Mono",
            monospace;
    }

    .success-copy {
        color: #86efac;
    }

    .error-copy {
        color: #fca5a5;
    }

    @media (max-width: 980px) {
        .setup-shell,
        .hero-panel,
        .footer-actions {
            grid-template-columns: 1fr;
        }
    }

    @media (max-width: 720px) {
        .form-grid,
        .review-grid {
            grid-template-columns: 1fr;
        }
    }
</style>
