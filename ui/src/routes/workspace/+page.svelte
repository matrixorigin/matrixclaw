<script lang="ts">
    import { errorMessage, fetchJson } from "$lib/http";
    import {
        queueControlsCopy,
        queueDeliveryLabels,
        type QueueControlKind,
        type QueueControlsView
    } from "$lib/queue";
    import type { ExecutionBackendLabel } from "$lib/execution";
    import { workspaceExplorerContract, type WorkspaceEntry } from "$lib/workspace";
    import { onMount, tick } from "svelte";

    type ExecutionSnapshot = {
        modeLabel: string;
        visibleBackends: ExecutionBackendLabel[];
        sandboxPriority: ExecutionBackendLabel[];
        sandboxFailureMessage: string;
        fallbackPolicy: string;
    };

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

    type ApiExecutionSnapshot = {
        modeLabel: string;
        visibleBackends: ExecutionBackendLabel[];
        sandboxPriority: ExecutionBackendLabel[];
        sandboxFailureMessage: string;
        fallbackPolicy: string;
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

    let workspaceEntries: WorkspaceEntry[] = [];
    let queueView: QueueControlsView | null = null;
    let executionSnapshot: ExecutionSnapshot | null = null;
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
    let sessionId = "";
    let streamPreviousEntries: TranscriptEntry[] = [];

    onMount(async () => {
        sessionId = createSessionId();
        await loadPage();
    });

    function createSessionId() {
        if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
            return crypto.randomUUID();
        }

        return `workspace-${Date.now()}`;
    }

    function queueStateRoute() {
        if (!sessionId.trim()) {
            return "/api/queue/state";
        }

        return `/api/queue/state?session_id=${encodeURIComponent(sessionId)}`;
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

    function setInitialTranscript(view: QueueControlsView, execution: ExecutionSnapshot) {
        transcriptEntries = [
            {
                role: "assistant",
                text: `Workspace shell is connected. ${view.steering.summary}`
            },
            {
                role: "tool",
                text: `Execution policy is ${execution.fallbackPolicy}. Preferred sandbox order is ${execution.sandboxPriority.join(", ")}.`,
                backend: execution.visibleBackends[0]
            },
            {
                role: "warning",
                text: execution.sandboxFailureMessage
            }
        ];
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
            sessionId = frame.session_id.trim() || sessionId;
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
            const [entryPayload, queuePayload, executionPayload] = await Promise.all([
                fetchJson<ApiWorkspaceEntry[]>(workspaceExplorerContract.filesRoute),
                fetchJson<ApiQueueControlsView>(queueStateRoute()),
                fetchJson<ApiExecutionSnapshot>("/api/execution/visibility")
            ]);

            workspaceEntries = entryPayload.map(normalizeWorkspaceEntry);
            queueView = normalizeQueueView(queuePayload);
            executionSnapshot = executionPayload;
            composerReferences = workspaceEntries
                .filter((entry) => entry.kind === "file")
                .slice(0, 2)
                .map((entry) => entry.referenceToken);
            promptDraft =
                composerReferences.length >= 2
                    ? `Review ${composerReferences[0]} before touching ${composerReferences[1]}.`
                    : "";
            setInitialTranscript(queueView, executionSnapshot);
        } catch (error) {
            pageError = errorMessage(error);
        } finally {
            loading = false;
        }
    }

    async function attachReference(entry: WorkspaceEntry) {
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

            if (!composerReferences.includes(response.reference_token)) {
                composerReferences = [...composerReferences, response.reference_token];
            }

            promptDraft = `${promptDraft}\n${response.reference_token}`.trim();
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
                    session_id: sessionId || undefined
                })
            });
            sessionId = response.session_id?.trim() || sessionId;

            const queuePayload = await fetchJson<ApiQueueControlsView>(queueStateRoute());
            queueView = normalizeQueueView(queuePayload);
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
                    session_id: sessionId || undefined
                })
            });
            await consumeStreamResponse(response);
            queueView = normalizeQueueView(await fetchJson<ApiQueueControlsView>(queueStateRoute()));
        } catch (error) {
            pageError = errorMessage(error);
        } finally {
            busy = false;
        }
    }
</script>

<section class="workspace-shell">
    <aside class="left-rail">
        <div class="panel-heading">
            <p class="section-label">Workspace</p>
            <h2>Files and references</h2>
        </div>

        <div class="contract-card">
            <span>List route</span>
            <code>{workspaceExplorerContract.filesRoute}</code>
            <span>Reference route</span>
            <code>{workspaceExplorerContract.referenceRoute}</code>
        </div>

        {#if loading}
            <p class="status-copy">Loading workspace…</p>
        {:else if pageError}
            <p class="error-copy">{pageError}</p>
        {:else}
            <ul class="file-list">
                {#each workspaceEntries as entry}
                    <li>
                        <div>
                            <strong>{entry.relativePath}</strong>
                            <small>
                                {entry.kind === "directory" ? "Directory" : entry.referenceToken}
                            </small>
                        </div>
                        <button
                            type="button"
                            disabled={busy || entry.kind === "directory"}
                            on:click={() => attachReference(entry)}
                        >
                            {entry.kind === "directory" ? "Browse" : "Reference"}
                        </button>
                    </li>
                {/each}
            </ul>
        {/if}
    </aside>

    <div class="main-column">
        <div class="transcript">
            <div class="panel-heading">
                <p class="section-label">Transcript</p>
                <h2>Loopback run stream</h2>
            </div>

            {#each transcriptEntries as item}
                {#if item.role === "assistant"}
                    <article data-role="assistant"><p>{item.text}</p></article>
                {:else}
                    <article data-role={item.role}>
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

        {#if queueView}
            <section class="queue-strip">
                <div class="queue-card">
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
                </div>

                <div class="queue-card">
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
                </div>
            </section>
        {/if}

        <form class="composer">
            <label for="prompt">Composer</label>
            <textarea
                id="prompt"
                rows="4"
                bind:value={promptDraft}
                placeholder="Ask the agent, attach references, or queue a steering update."
            ></textarea>
            <div class="composer-actions">
                <div class="reference-chips">
                    {#each composerReferences as reference}
                        <span>{reference}</span>
                    {/each}
                </div>
                <button type="button" disabled={busy || !promptDraft.trim()} on:click={sendPrompt}>
                    Send
                </button>
            </div>
        </form>
    </div>

    <aside class="right-rail">
        <div class="panel-heading">
            <p class="section-label">Run state</p>
            <h2>Queue and execution detail</h2>
        </div>

        <div class="execution-card">
            <span class="card-title">Visible backends</span>
            <div class="backend-stack">
                {#each executionSnapshot?.visibleBackends ?? [] as backend}
                    <span class="backend-badge">{backend}</span>
                {/each}
            </div>
            <p>Sandbox priority</p>
            <ol>
                {#each executionSnapshot?.sandboxPriority ?? [] as backend, index}
                    <li>{index + 1}. {backend}</li>
                {/each}
            </ol>
        </div>

        <div class="execution-card failure">
            <span class="card-title">Sandbox policy</span>
            <strong>{executionSnapshot?.sandboxFailureMessage ?? "loading execution policy"}</strong>
            <p>
                Required-sandbox failures stay explicit instead of silently falling back to local.
            </p>
        </div>

        <div class="execution-card">
            <span class="card-title">Runtime contract</span>
            <p>Mode: <code>{executionSnapshot?.modeLabel ?? "loading"}</code></p>
            <p>Fallback: <code>{executionSnapshot?.fallbackPolicy ?? "loading"}</code></p>
        </div>
    </aside>
</section>

<style>
    .workspace-shell {
        display: grid;
        grid-template-columns: minmax(17rem, 22rem) minmax(0, 1fr) minmax(17rem, 22rem);
        gap: 1rem;
    }

    .left-rail,
    .right-rail,
    .transcript,
    .composer,
    .queue-card,
    .execution-card,
    .contract-card {
        padding: 1.1rem;
        border-radius: 1.25rem;
        border: 1px solid rgba(148, 163, 184, 0.16);
        background: rgba(15, 23, 42, 0.72);
    }

    .main-column {
        display: grid;
        gap: 1rem;
    }

    .panel-heading {
        margin-bottom: 0.85rem;
    }

    .section-label {
        margin: 0 0 0.4rem;
        color: #86efac;
        letter-spacing: 0.16em;
        text-transform: uppercase;
        font-size: 0.78rem;
    }

    h2,
    h3 {
        margin: 0;
    }

    .contract-card {
        display: grid;
        gap: 0.3rem;
        margin-bottom: 1rem;
    }

    .contract-card span,
    .execution-card p,
    .queue-card p,
    .queue-card small,
    article p,
    .status-copy {
        color: #cbd5e1;
        line-height: 1.55;
    }

    .error-copy {
        color: #fecaca;
        line-height: 1.55;
    }

    code {
        color: #fde68a;
        font-family:
            "IBM Plex Mono",
            monospace;
    }

    .file-list,
    ol {
        margin: 0;
        padding: 0;
        list-style: none;
    }

    .file-list li {
        display: grid;
        gap: 0.6rem;
        margin-bottom: 0.75rem;
        padding: 0.85rem;
        border-radius: 1rem;
        background: rgba(30, 41, 59, 0.66);
    }

    .file-list strong,
    .execution-card strong {
        color: #f8fafc;
    }

    .file-list small {
        display: block;
        margin-top: 0.25rem;
        color: #94a3b8;
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

    button:disabled {
        opacity: 0.55;
        cursor: not-allowed;
    }

    .transcript {
        min-height: 18rem;
    }

    article {
        margin-top: 0.85rem;
        padding: 0.95rem;
        border-radius: 1rem;
        background: rgba(2, 6, 23, 0.45);
    }

    article[data-role="tool"] {
        border-left: 3px solid #38bdf8;
    }

    article[data-role="warning"] {
        border-left: 3px solid #f87171;
    }

    .entry-header {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        align-items: center;
        margin-bottom: 0.4rem;
    }

    .backend-badge {
        display: inline-flex;
        align-items: center;
        padding: 0.25rem 0.65rem;
        border-radius: 999px;
        background: rgba(251, 191, 36, 0.16);
        color: #fde68a;
        font-size: 0.85rem;
        font-weight: 700;
        text-transform: lowercase;
    }

    .queue-strip {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 1rem;
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

    .queue-card textarea {
        min-height: 5rem;
    }

    .composer-actions {
        display: flex;
        flex-wrap: wrap;
        gap: 0.75rem;
        justify-content: space-between;
        align-items: center;
    }

    .reference-chips {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
    }

    .reference-chips span {
        padding: 0.35rem 0.6rem;
        border-radius: 999px;
        background: rgba(59, 130, 246, 0.14);
        color: #bfdbfe;
        font-family:
            "IBM Plex Mono",
            monospace;
        font-size: 0.85rem;
    }

    .execution-card {
        margin-bottom: 0.9rem;
    }

    .card-title {
        display: block;
        margin-bottom: 0.55rem;
        color: #93c5fd;
        font-size: 0.8rem;
        text-transform: uppercase;
        letter-spacing: 0.12em;
    }

    .backend-stack {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        margin-bottom: 0.75rem;
    }

    .failure {
        border-color: rgba(248, 113, 113, 0.3);
    }

    .failure p {
        margin-top: 0.45rem;
    }

    @media (max-width: 1080px) {
        .workspace-shell {
            grid-template-columns: 1fr;
        }
    }

    @media (max-width: 720px) {
        .queue-strip {
            grid-template-columns: 1fr;
        }

        .composer-actions {
            align-items: flex-start;
        }
    }
</style>
