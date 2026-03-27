<script lang="ts">
    import {
        visibleExecutionBackends,
        sandboxFailureMessage,
        sandboxPriority,
        type ExecutionBackendLabel
    } from "$lib/execution";
    import {
        queueControlsCopy,
        queueDeliveryLabels,
        type QueueControlsView,
        type QueueSubmissionRequest
    } from "$lib/queue";
    import {
        formatWorkspaceReference,
        workspaceExplorerContract,
        type WorkspaceEntry
    } from "$lib/workspace";

    type TranscriptEntry = {
        role: "assistant" | "tool" | "warning";
        text: string;
        backend?: ExecutionBackendLabel;
    };

    const workspaceEntries: WorkspaceEntry[] = [
        {
            relativePath: "agents/default/SOUL.md",
            kind: "file",
            referenceToken: formatWorkspaceReference("agents/default/SOUL.md")
        },
        {
            relativePath: "agents/default/MEMORY.md",
            kind: "file",
            referenceToken: formatWorkspaceReference("agents/default/MEMORY.md")
        },
        {
            relativePath: "workspace/specs/queue/notes.md",
            kind: "file",
            referenceToken: formatWorkspaceReference("workspace/specs/queue/notes.md")
        },
        {
            relativePath: "workspace/src",
            kind: "directory",
            referenceToken: formatWorkspaceReference("workspace/src")
        }
    ];

    const queueView: QueueControlsView = {
        steering: {
            kind: "steering",
            submitRoute: "/api/queue/steering",
            deliveryTiming: "next-turn",
            summary: "1 steering item queued for the next assistant turn"
        },
        followUp: {
            kind: "follow-up",
            submitRoute: "/api/queue/follow-up",
            deliveryTiming: "next-run",
            summary: "1 follow-up item deferred until the current run completes"
        }
    };

    const queuedDrafts: QueueSubmissionRequest[] = [
        {
            kind: "steering",
            message: "Prefer the workspace file reference instead of pasting file contents."
        },
        {
            kind: "follow-up",
            message: "After the current run, open the Skills page and enable the deploy skill."
        }
    ];

    const transcriptEntries: TranscriptEntry[] = [
        {
            role: "assistant",
            text: "Workspace shell is reading files and preparing the next tool call.",
            backend: "local"
        },
        {
            role: "tool",
            text: "Queued code execution completed in a Docker sandbox with deterministic mounts.",
            backend: "docker"
        },
        {
            role: "warning",
            text: sandboxFailureMessage
        }
    ];

    const composerReferences = workspaceEntries
        .filter((entry) => entry.kind === "file")
        .slice(0, 2)
        .map((entry) => entry.referenceToken);
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

        <ul class="file-list">
            {#each workspaceEntries as entry}
                <li>
                    <div>
                        <strong>{entry.relativePath}</strong>
                        <small>{entry.kind === "directory" ? "Directory" : entry.referenceToken}</small>
                    </div>
                    <button type="button">
                        {entry.kind === "directory" ? "Browse" : "Reference"}
                    </button>
                </li>
            {/each}
        </ul>
    </aside>

    <div class="main-column">
        <div class="transcript">
            <div class="panel-heading">
                <p class="section-label">Transcript</p>
                <h2>Run stream</h2>
            </div>

            {#each transcriptEntries as item}
                <article data-role={item.role}>
                    <div class="entry-header">
                        <strong>{item.role}</strong>
                        {#if item.backend}
                            <span class="backend-badge">{item.backend}</span>
                        {/if}
                    </div>
                    <p>{item.text}</p>
                </article>
            {/each}
        </div>

        <section class="queue-strip">
            <div class="queue-card">
                <p class="section-label">Steering</p>
                <h3>{queueDeliveryLabels[queueView.steering.deliveryTiming]}</h3>
                <p>{queueControlsCopy.steering}</p>
                <small>{queueView.steering.summary}</small>
            </div>

            <div class="queue-card">
                <p class="section-label">Follow-up</p>
                <h3>{queueDeliveryLabels[queueView.followUp.deliveryTiming]}</h3>
                <p>{queueControlsCopy.followUp}</p>
                <small>{queueView.followUp.summary}</small>
            </div>
        </section>

        <form class="composer">
            <label for="prompt">Composer</label>
            <textarea
                id="prompt"
                rows="4"
                placeholder="Ask the agent, attach references, or queue a steering update."
            >Review {composerReferences[0]} before touching {composerReferences[1]}.</textarea>
            <div class="composer-actions">
                <div class="reference-chips">
                    {#each composerReferences as reference}
                        <span>{reference}</span>
                    {/each}
                </div>
                <button type="button">Send</button>
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
                {#each visibleExecutionBackends as backend}
                    <span class="backend-badge">{backend}</span>
                {/each}
            </div>
            <p>Sandbox priority</p>
            <ol>
                {#each sandboxPriority as backend, index}
                    <li>{index + 1}. {backend}</li>
                {/each}
            </ol>
        </div>

        <div class="execution-card failure">
            <span class="card-title">Sandbox policy</span>
            <strong>{sandboxFailureMessage}</strong>
            <p>Required-sandbox failures stay explicit instead of silently falling back to local.</p>
        </div>

        <div class="execution-card">
            <span class="card-title">Queued drafts</span>
            <ul class="draft-list">
                {#each queuedDrafts as draft}
                    <li>
                        <strong>{draft.kind}</strong>
                        <small>{draft.message}</small>
                    </li>
                {/each}
            </ul>
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
    .draft-list small,
    article p {
        color: #cbd5e1;
        line-height: 1.55;
    }

    code {
        color: #fde68a;
        font-family:
            "IBM Plex Mono",
            monospace;
    }

    .file-list,
    .draft-list,
    ol {
        margin: 0;
        padding: 0;
        list-style: none;
    }

    .file-list li,
    .draft-list li {
        display: grid;
        gap: 0.6rem;
        margin-bottom: 0.75rem;
        padding: 0.85rem;
        border-radius: 1rem;
        background: rgba(30, 41, 59, 0.66);
    }

    .file-list strong,
    .draft-list strong,
    .execution-card strong {
        color: #f8fafc;
    }

    .file-list small,
    .draft-list small {
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
