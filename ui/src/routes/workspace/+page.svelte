<script lang="ts">
    import { onMount, tick } from "svelte";

    import { agentRoute, fetchAgent } from "$lib/agents";
    import { createSelectedAgentSession, displaySelectedAgentName, type SelectedAgentSession } from "$lib/agents/session";
    import { errorMessage, fetchJson } from "$lib/http";
    import {
        queueControlsCopy,
        queueDeliveryLabels,
        type QueueControlKind,
        type QueueControlsView
    } from "$lib/queue";
    import { workspaceExplorerContract, type WorkspaceEntry } from "$lib/workspace";
    import {
        buildWorkspaceDockModel,
        type WorkspaceDockModel
    } from "$lib/workspace/dock";
    import {
        buildWorkspaceAgentSurface,
        buildWorkspaceShellDiagnostics,
        type WorkspaceAgentSurface,
        type WorkspaceExecutionSnapshot,
        type WorkspaceShellDiagnostics
    } from "$lib/workspace/shell";

    type TranscriptEntry = {
        role: "assistant" | "tool" | "warning";
        text: string;
        backend?: string;
    };

    type ApiWorkspaceEntry = {
        relative_path: string;
        kind: "File" | "Directory";
        reference_token: string;
    };

    type ApiWorkspaceReferenceResponse = {
        relative_path: string;
        reference_token: string;
    };

    type ApiQueueControlState = {
        kind: QueueControlKind;
        submit_route: string;
        delivery_timing: "next-turn" | "next-run" | "queued";
        summary: string;
    };

    type ApiQueueControlsView = {
        steering: ApiQueueControlState;
        follow_up: ApiQueueControlState;
    };

    type ApiQueueSubmissionResult = {
        accepted: boolean;
        session_id: string;
        state: ApiQueueControlState;
    };

    type AgentRunEvent = {
        sequence: number;
        kind: string;
        content?: string | null;
    };

    type AgentRunStreamFrame =
        | {
              type: "event";
              event: AgentRunEvent;
          }
        | {
              type: "complete";
              session_id: string;
              model: string;
              streamed_message: string;
              final_message: string;
          }
        | {
              type: "error";
              error: string;
          };

    let selectedAgentSession: SelectedAgentSession = createSelectedAgentSession();
    let activeAgentSurface: WorkspaceAgentSurface | null = null;
    let workspaceDock: WorkspaceDockModel = buildWorkspaceDockModel(
        selectedAgentSession.agentName,
        activeAgentSurface
    );
    let workspaceEntries: WorkspaceEntry[] = [];
    let queueView: QueueControlsView | null = null;
    let executionSnapshot: WorkspaceExecutionSnapshot | null = null;
    let shellDiagnostics: WorkspaceShellDiagnostics | null = null;
    let transcriptEntries: TranscriptEntry[] = [];
    let composerReferences: string[] = [];
    let steeringDraft =
        "Prefer the workspace file reference instead of pasting file contents.";
    let followUpDraft =
        "After the current run, open the Skills page and enable the lint-bridge skill.";
    let promptDraft = "";
    let loading = true;
    let busy = false;
    let pageError = "";
    let streamPreviousEntries: TranscriptEntry[] = [];
    let activeAgentLabel = displaySelectedAgentName(selectedAgentSession.agentName);
    let composerPlaceholder = `Message ${activeAgentLabel}...`;

    onMount(() => {
        if (!selectedAgentSession.sessionId.trim()) {
            setSessionId(createSessionId());
        }
        void loadPage();
    });

    $: activeAgentLabel = displaySelectedAgentName(selectedAgentSession.agentName);
    $: composerPlaceholder = `Message ${activeAgentLabel}...`;
    $: workspaceDock = buildWorkspaceDockModel(
        selectedAgentSession.agentName,
        activeAgentSurface
    );

    function createSessionId() {
        if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
            return crypto.randomUUID();
        }

        return `workspace-${Date.now()}`;
    }

    function queueStateRoute() {
        if (!selectedAgentSession.sessionId.trim()) {
            return "/api/queue/state";
        }

        return `/api/queue/state?session_id=${encodeURIComponent(selectedAgentSession.sessionId)}`;
    }

    function normalizeWorkspaceEntry(entry: ApiWorkspaceEntry): WorkspaceEntry {
        return {
            relativePath: entry.relative_path,
            kind: entry.kind === "Directory" ? "directory" : "file",
            referenceToken: entry.reference_token
        };
    }

    function normalizeQueueView(view: ApiQueueControlsView): QueueControlsView {
        return {
            steering: {
                kind: view.steering.kind,
                submitRoute: view.steering.submit_route,
                deliveryTiming: view.steering.delivery_timing,
                summary: view.steering.summary
            },
            followUp: {
                kind: view.follow_up.kind,
                submitRoute: view.follow_up.submit_route,
                deliveryTiming: view.follow_up.delivery_timing,
                summary: view.follow_up.summary
            }
        };
    }

    function setInitialTranscript() {
        transcriptEntries = [
            {
                role: "assistant",
                text: `Connected to ${activeAgentSurface?.heading ?? activeAgentLabel}. Use the composer to message the active agent, attach references, and inspect run state.`
            }
        ];
    }

    function setSessionId(sessionId: string) {
        selectedAgentSession = createSelectedAgentSession(selectedAgentSession.agentName, sessionId);
    }

    function appendReference(referenceToken: string) {
        if (!composerReferences.includes(referenceToken)) {
            composerReferences = [...composerReferences, referenceToken];
        }

        promptDraft = promptDraft.trim()
            ? `${promptDraft.trim()}\n${referenceToken}`
            : referenceToken;
    }

    function startStreamShell(modelLabel: string) {
        transcriptEntries = [
            {
                role: "assistant",
                text: ""
            },
            {
                role: "tool",
                text: `Provider model: ${modelLabel.trim() || "streaming"}`
            },
            ...streamPreviousEntries
        ];
    }

    function setAssistantText(text: string) {
        if (transcriptEntries.length === 0) {
            return;
        }

        transcriptEntries = [
            {
                ...transcriptEntries[0],
                text
            },
            ...transcriptEntries.slice(1)
        ];
    }

    function updateModelLabel(modelLabel: string) {
        if (transcriptEntries.length < 2) {
            return;
        }

        transcriptEntries = [
            transcriptEntries[0],
            {
                ...transcriptEntries[1],
                text: `Provider model: ${modelLabel.trim() || "unknown-model"}`
            },
            ...transcriptEntries.slice(2)
        ];
    }

    async function renderAssistantFrame(frame: AgentRunStreamFrame) {
        if (frame.type === "error") {
            throw new Error(frame.error);
        }

        if (frame.type === "complete") {
            setSessionId(frame.session_id.trim() || selectedAgentSession.sessionId);
            startStreamShell(frame.model);
            updateModelLabel(frame.model);
            setAssistantText(frame.final_message.trim() || frame.streamed_message.trim() || "");
            await tick();
            return;
        }

        const event = frame.event;
        if (transcriptEntries.length === 0) {
            startStreamShell("streaming");
        }

        if (event.kind === "message_started") {
            setAssistantText("");
        } else if (event.kind === "message_delta") {
            const current = transcriptEntries[0]?.text ?? "";
            setAssistantText(`${current}${event.content ?? ""}`);
        } else if (event.kind === "message_completed") {
            setAssistantText(event.content?.trim() || transcriptEntries[0]?.text || "");
        }

        await tick();
    }

    async function consumeStreamResponse(response: Response) {
        if (!response.ok) {
            const body = await response.text();
            throw new Error(body || `request failed with status ${response.status}`);
        }

        if (!response.body) {
            throw new Error("stream response did not include a body");
        }

        const reader = response.body.getReader();
        const decoder = new TextDecoder();
        let buffer = "";

        try {
            while (true) {
                const { value, done } = await reader.read();
                if (value) {
                    buffer += decoder.decode(value, { stream: !done });
                }

                let splitIndex = buffer.indexOf("\n\n");
                while (splitIndex !== -1) {
                    const block = buffer.slice(0, splitIndex);
                    buffer = buffer.slice(splitIndex + 2);
                    const payload = extractSsePayload(block);
                    if (payload) {
                        await renderAssistantFrame(JSON.parse(payload) as AgentRunStreamFrame);
                    }
                    splitIndex = buffer.indexOf("\n\n");
                }

                if (done) {
                    break;
                }
            }

            buffer += decoder.decode();
            const payload = extractSsePayload(buffer);
            if (payload) {
                await renderAssistantFrame(JSON.parse(payload) as AgentRunStreamFrame);
            }
        } finally {
            reader.releaseLock();
        }
    }

    function extractSsePayload(block: string): string | null {
        const payload = block
            .split(/\r?\n/)
            .map((line) => line.trim())
            .filter((line) => line.startsWith("data:"))
            .map((line) => line.slice(5).trimStart())
            .join("\n")
            .trim();

        return payload ? payload : null;
    }

    async function loadPage() {
        loading = true;
        pageError = "";

        try {
            const [workspacePayload, agentPayload, queuePayload, executionPayload] = await Promise.all([
                fetchJson<ApiWorkspaceEntry[]>(workspaceExplorerContract.filesRoute),
                fetchAgent(selectedAgentSession.agentName),
                fetchJson<ApiQueueControlsView>(queueStateRoute()),
                fetchJson<WorkspaceExecutionSnapshot>("/api/execution/visibility")
            ]);

            workspaceEntries = workspacePayload.map(normalizeWorkspaceEntry);
            activeAgentSurface = buildWorkspaceAgentSurface(agentPayload);
            workspaceDock = buildWorkspaceDockModel(
                selectedAgentSession.agentName,
                activeAgentSurface
            );
            queueView = normalizeQueueView(queuePayload);
            executionSnapshot = executionPayload;
            shellDiagnostics = buildWorkspaceShellDiagnostics(queueView, executionSnapshot);
            composerReferences = [];
            promptDraft = "";
            setInitialTranscript();
        } catch (error) {
            pageError = errorMessage(error);
        } finally {
            loading = false;
        }
    }

    async function attachReference(entry: WorkspaceEntry) {
        if (entry.kind === "directory") {
            return;
        }

        busy = true;
        pageError = "";

        try {
            const response = await fetchJson<ApiWorkspaceReferenceResponse>(
                workspaceExplorerContract.referenceRoute,
                {
                    method: "POST",
                    body: JSON.stringify({
                        relative_path: entry.relativePath
                    })
                }
            );

            appendReference(response.reference_token);
            transcriptEntries = [
                {
                    role: "assistant",
                    text: `Attached workspace reference ${response.reference_token}.`
                },
                ...transcriptEntries
            ];
        } catch (error) {
            pageError = errorMessage(error);
        } finally {
            busy = false;
        }
    }

    async function submitQueue(kind: QueueControlKind, message: string) {
        busy = true;
        pageError = "";

        try {
            const route = kind === "steering" ? "/api/queue/steering" : "/api/queue/follow-up";
            const response = await fetchJson<ApiQueueSubmissionResult>(route, {
                method: "POST",
                body: JSON.stringify({
                    kind,
                    message,
                    session_id: selectedAgentSession.sessionId || undefined
                })
            });

            setSessionId(response.session_id.trim() || selectedAgentSession.sessionId);

            const queuePayload = await fetchJson<ApiQueueControlsView>(queueStateRoute());
            queueView = normalizeQueueView(queuePayload);
            if (executionSnapshot) {
                shellDiagnostics = buildWorkspaceShellDiagnostics(queueView, executionSnapshot);
            }
            transcriptEntries = [
                {
                    role: "assistant",
                    text: `${kind === "steering" ? "Steering" : "Follow-up"} queued: ${message}`
                },
                ...transcriptEntries
            ];
        } catch (error) {
            pageError = errorMessage(error);
        } finally {
            busy = false;
        }
    }

    async function sendPrompt() {
        if (!promptDraft.trim()) {
            return;
        }

        busy = true;
        pageError = "";

        try {
            streamPreviousEntries = transcriptEntries;
            startStreamShell("streaming");
            const response = await fetch("/api/agent/run/stream", {
                method: "POST",
                headers: {
                    "content-type": "application/json"
                },
                body: JSON.stringify({
                    prompt: promptDraft.trim(),
                    session_id: selectedAgentSession.sessionId || undefined,
                    agent_name: selectedAgentSession.agentName
                })
            });
            await consumeStreamResponse(response);
            const queuePayload = await fetchJson<ApiQueueControlsView>(queueStateRoute());
            queueView = normalizeQueueView(queuePayload);
            if (executionSnapshot) {
                shellDiagnostics = buildWorkspaceShellDiagnostics(queueView, executionSnapshot);
            }
        } catch (error) {
            pageError = errorMessage(error);
        } finally {
            busy = false;
        }
    }
</script>

<svelte:head>
    <title>Workspace | MatrixClaw</title>
</svelte:head>

<section class="workspace-shell">
    <aside class="left-rail">
        <section class="surface-card agent-card">
            <p class="section-label">Active Agent</p>
            <h2>{activeAgentSurface?.heading ?? activeAgentLabel}</h2>
            <p class="lead">{workspaceDock.crownJobSummary}</p>

            <div class="detail-grid">
                <article class="detail-card">
                    <p class="section-label">Crown Job</p>
                    <p>{workspaceDock.crownJobSummary}</p>
                </article>

                <article class="detail-card">
                    <p class="section-label">Memory</p>
                    <p>{activeAgentSurface?.memorySummary ?? "Loading memory summary..."}</p>
                </article>
            </div>

            <dl class="metric-grid">
                {#each workspaceDock.agentState as row}
                    <div>
                        <dt>{row.label}</dt>
                        <dd>{row.value}</dd>
                    </div>
                {/each}
                {#each workspaceDock.capabilityState as row}
                    <div>
                        <dt>{row.label}</dt>
                        <dd>{row.value}</dd>
                    </div>
                {/each}
                <div>
                    <dt>Session</dt>
                    <dd>{selectedAgentSession.sessionId || "new session"}</dd>
                </div>
            </dl>

            <div class="chip-row">
                {#if activeAgentSurface}
                    {#each activeAgentSurface.enabledSkills as skill}
                        <span>{skill}</span>
                    {/each}
                    {#each activeAgentSurface.enabledMcpServers as server}
                        <span>{server}</span>
                    {/each}
                    {#each activeAgentSurface.enabledGateways as gateway}
                        <span>{gateway}</span>
                    {/each}
                {/if}
            </div>

            <a class="agent-link" href={agentRoute(selectedAgentSession.agentName)}>
                Open agent configuration
            </a>
        </section>
    </aside>

    <section class="main-column">
        <div class="panel-heading">
            <p class="section-label">Conversation</p>
            <h2>Conversation</h2>
            <p class="lead">
                Talk to the selected agent, attach workspace references, and keep the transcript
                grounded.
            </p>
        </div>

        {#if pageError}
            <p class="error-copy" role="alert">{pageError}</p>
        {/if}

        {#if loading}
            <p class="state-copy">Loading workspace...</p>
        {/if}

        <div class="transcript-stack">
            {#each transcriptEntries as item}
                {#if item.role === "assistant"}
                    <article class="transcript-card assistant" data-role={item.role}>
                        <p>{item.text}</p>
                    </article>
                {:else}
                    <article class={`transcript-card ${item.role}`} data-role={item.role}>
                        <div class="entry-header">
                            <strong>{item.role}</strong>
                            {#if item.backend}
                                <span class="backend-badge">{item.backend}</span>
                            {/if}
                        </div>
                        <p>{item.text}</p>
                    </article>
                {/if}
            {/each}
        </div>

        <form class="composer" on:submit|preventDefault={sendPrompt}>
            <label for="prompt">Composer</label>
            <textarea
                id="prompt"
                rows="4"
                bind:value={promptDraft}
                placeholder={composerPlaceholder}
            ></textarea>

            <section class="reference-tray">
                <div class="tray-header">
                    <p class="section-label">Reference tray</p>
                    <span>{workspaceEntries.length} entries</span>
                </div>

                {#if workspaceEntries.length > 0}
                    <div class="reference-list">
                        {#each workspaceEntries as entry}
                            <article class="reference-row">
                                <div>
                                    <strong>{entry.relativePath}</strong>
                                    <small>
                                        {entry.kind === "directory"
                                            ? "Directory"
                                            : entry.referenceToken}
                                    </small>
                                </div>
                                <button
                                    type="button"
                                    disabled={busy || entry.kind === "directory"}
                                    on:click={() => attachReference(entry)}
                                >
                                    {entry.kind === "directory" ? "Browse" : "Reference"}
                                </button>
                            </article>
                        {/each}
                    </div>
                {:else}
                    <p class="state-copy">No workspace references available.</p>
                {/if}
            </section>

            <div class="composer-footer">
                <div class="reference-chips">
                    {#if composerReferences.length > 0}
                        {#each composerReferences as reference}
                            <button
                                type="button"
                                class="reference-chip"
                                on:click={() => appendReference(reference)}
                            >
                                {reference}
                            </button>
                        {/each}
                    {:else}
                        <p class="state-copy">Select references from the tray above.</p>
                    {/if}
                </div>

                <button
                    type="button"
                    class="send-button"
                    disabled={busy || !promptDraft.trim()}
                    on:click={sendPrompt}
                >
                    Send
                </button>
            </div>
        </form>
    </section>

    <aside class="right-rail">
        <div class="panel-heading">
            <p class="section-label">Run State</p>
            <h2>Run State</h2>
            <p class="lead">
                Live queue posture and execution policy for the active session.
            </p>
        </div>

        <div class="state-stack">
            {#if shellDiagnostics}
                {#each shellDiagnostics.queueCards as card}
                    <article class:warning={card.tone === "warning"} class="summary-card">
                        <span class="card-title">{card.title}</span>
                        <strong>{card.label}</strong>
                        <p>{card.body}</p>
                    </article>
                {/each}
                {#each shellDiagnostics.executionCards as card}
                    <article class:warning={card.tone === "warning"} class="summary-card">
                        <span class="card-title">{card.title}</span>
                        <strong>{card.label}</strong>
                        <p>{card.body}</p>
                    </article>
                {/each}
            {/if}
        </div>

        {#if queueView}
            <section class="queue-controls">
                <article class="queue-card">
                    <p class="section-label">Steering</p>
                    <h3>{queueDeliveryLabels[queueView.steering.deliveryTiming]}</h3>
                    <p>{queueControlsCopy.steering}</p>
                    <small>{queueView.steering.summary}</small>
                    <textarea
                        rows="3"
                        bind:value={steeringDraft}
                        placeholder="Queue the next-turn steering instruction."
                    ></textarea>
                    <button
                        type="button"
                        disabled={busy || !steeringDraft.trim()}
                        on:click={() => submitQueue("steering", steeringDraft.trim())}
                    >
                        Queue steering
                    </button>
                </article>

                <article class="queue-card">
                    <p class="section-label">Follow-up</p>
                    <h3>{queueDeliveryLabels[queueView.followUp.deliveryTiming]}</h3>
                    <p>{queueControlsCopy.followUp}</p>
                    <small>{queueView.followUp.summary}</small>
                    <textarea
                        rows="3"
                        bind:value={followUpDraft}
                        placeholder="Queue the post-run follow-up instruction."
                    ></textarea>
                    <button
                        type="button"
                        disabled={busy || !followUpDraft.trim()}
                        on:click={() => submitQueue("follow-up", followUpDraft.trim())}
                    >
                        Queue follow-up
                    </button>
                </article>
            </section>
        {/if}
    </aside>
</section>

<style>
    .workspace-shell {
        display: grid;
        grid-template-columns: minmax(18rem, 22rem) minmax(0, 1fr) minmax(18rem, 22rem);
        gap: 1rem;
        align-items: start;
    }

    .left-rail,
    .right-rail,
    .surface-card,
    .transcript-card,
    .composer,
    .queue-card,
    .summary-card,
    .reference-row,
    .detail-card {
        border: 1px solid var(--mc-border);
        border-radius: var(--mc-radius-card);
        background: var(--mc-surface);
        box-shadow: 0 12px 24px rgba(27, 34, 51, 0.06);
    }

    .left-rail,
    .right-rail,
    .main-column {
        display: grid;
        gap: 1rem;
        min-width: 0;
    }

    .surface-card,
    .composer,
    .queue-card,
    .summary-card,
    .transcript-card {
        padding: 1rem 1.1rem;
    }

    .panel-heading {
        display: grid;
        gap: 0.4rem;
    }

    .section-label {
        margin: 0;
        color: var(--mc-primary);
        font-size: 0.78rem;
        letter-spacing: 0.16em;
        text-transform: uppercase;
    }

    h2,
    h3,
    p,
    dl {
        margin: 0;
    }

    h2 {
        color: var(--mc-text);
        font-size: clamp(1.6rem, 3vw, 2.3rem);
        line-height: 1;
    }

    h3 {
        color: var(--mc-text);
        font-size: 1.05rem;
    }

    .lead,
    .state-copy,
    .error-copy,
    .transcript-card p,
    .queue-card p,
    .queue-card small,
    .reference-row small {
        color: var(--mc-text-secondary);
        line-height: 1.55;
    }

    .error-copy {
        color: #b91c1c;
    }

    .agent-card {
        display: grid;
        gap: 1rem;
    }

    .detail-grid,
    .metric-grid,
    .state-stack,
    .reference-list,
    .reference-chips {
        display: grid;
        gap: 0.75rem;
    }

    .detail-grid {
        grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .detail-card {
        padding: 0.85rem 0.95rem;
        display: grid;
        gap: 0.45rem;
    }

    .detail-card p {
        color: var(--mc-text-secondary);
        line-height: 1.5;
    }

    .metric-grid {
        grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .metric-grid div {
        display: grid;
        gap: 0.12rem;
        padding: 0.7rem 0.8rem;
        border-radius: var(--mc-radius-input);
        background: var(--mc-raised);
        border: 1px solid var(--mc-border);
    }

    dt {
        color: var(--mc-text-muted);
        font-size: 0.76rem;
        letter-spacing: 0.08em;
        text-transform: uppercase;
    }

    dd {
        margin: 0;
        color: var(--mc-text);
        font-weight: 600;
    }

    .chip-row,
    .reference-chips {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
    }

    .chip-row span,
    .reference-chip,
    .backend-badge,
    .tray-header span,
    .card-title {
        display: inline-flex;
        align-items: center;
        padding: 0.3rem 0.55rem;
        border-radius: 999px;
        background: rgba(91, 192, 235, 0.12);
        color: var(--mc-text);
        font-size: 0.84rem;
    }

    .agent-link {
        justify-self: start;
        padding: 0.6rem 0.9rem;
        border-radius: 999px;
        background: var(--mc-primary);
        color: #fff;
        font-weight: 600;
    }

    .transcript-stack {
        display: grid;
        gap: 0.75rem;
        min-height: 10rem;
    }

    .transcript-card {
        display: grid;
        gap: 0.55rem;
        background: linear-gradient(180deg, var(--mc-surface), var(--mc-raised));
    }

    .transcript-card.assistant {
        border-left: 3px solid var(--mc-primary);
    }

    .transcript-card.tool {
        border-left: 3px solid var(--mc-warning);
    }

    .transcript-card.warning {
        border-left: 3px solid var(--mc-danger);
    }

    .entry-header {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        align-items: center;
    }

    .backend-badge {
        background: rgba(245, 158, 11, 0.16);
        color: #92400e;
        text-transform: lowercase;
    }

    .composer {
        display: grid;
        gap: 0.9rem;
    }

    .composer label {
        color: var(--mc-text);
        font-weight: 600;
    }

    textarea {
        width: 100%;
        resize: vertical;
        min-height: 7rem;
        padding: 0.95rem 1rem;
        border: 1px solid var(--mc-border);
        border-radius: var(--mc-radius-input);
        background: var(--mc-raised);
        color: var(--mc-text);
        font: inherit;
    }

    .reference-tray {
        display: grid;
        gap: 0.7rem;
    }

    .tray-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 1rem;
    }

    .reference-list {
        gap: 0.6rem;
    }

    .reference-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 0.8rem;
        padding: 0.85rem 0.95rem;
        background: var(--mc-raised);
    }

    .reference-row strong {
        display: block;
        color: var(--mc-text);
    }

    .reference-row small {
        display: block;
        margin-top: 0.2rem;
    }

    .reference-row button,
    .send-button,
    .queue-card button {
        border: 0;
        border-radius: 999px;
        background: var(--mc-primary);
        color: #fff;
        font-weight: 600;
        cursor: pointer;
    }

    .reference-row button,
    .queue-card button {
        padding: 0.5rem 0.8rem;
    }

    .reference-row button:disabled,
    .send-button:disabled,
    .queue-card button:disabled {
        opacity: 0.55;
        cursor: not-allowed;
    }

    .composer-footer {
        display: flex;
        flex-wrap: wrap;
        justify-content: space-between;
        align-items: center;
        gap: 0.8rem;
    }

    .reference-chip {
        border: 0;
        cursor: pointer;
    }

    .send-button {
        padding: 0.7rem 1rem;
    }

    .state-stack {
        gap: 0.75rem;
    }

    .summary-card {
        display: grid;
        gap: 0.45rem;
    }

    .summary-card.warning {
        border-color: rgba(239, 68, 68, 0.3);
        background: rgba(254, 242, 242, 0.96);
    }

    .queue-controls {
        display: grid;
        gap: 0.75rem;
    }

    .queue-card {
        display: grid;
        gap: 0.55rem;
    }

    .queue-card textarea {
        min-height: 5rem;
    }

    .queue-card button {
        justify-self: start;
    }

    @media (max-width: 1180px) {
        .workspace-shell {
            grid-template-columns: 1fr;
        }

        .detail-grid,
        .metric-grid {
            grid-template-columns: 1fr 1fr;
        }
    }

    @media (max-width: 720px) {
        .detail-grid,
        .metric-grid {
            grid-template-columns: 1fr;
        }

        .composer-footer,
        .tray-header,
        .composer-footer .reference-chips {
            width: 100%;
        }

        .send-button,
        .queue-card button,
        .reference-row button {
            width: 100%;
        }

        .reference-row {
            align-items: flex-start;
            flex-direction: column;
        }
    }
</style>
