<section class="placeholder-shell">
    <div class="placeholder-card">
        <p class="section-label">Agents</p>
        <h2>Agent directory is moving into the new product shell</h2>
        <p>
            This route now exists so the shared shell navigation stays truthful while we build the
            full per-agent directory and configuration surfaces.
        </p>
    </div>
</section>

<style>
    .placeholder-shell {
        min-height: 100%;
        display: grid;
    }

    .placeholder-card {
        display: grid;
        gap: 0.9rem;
        align-content: start;
        padding: 1.4rem;
        border: 1px solid var(--mc-border);
        border-radius: var(--mc-radius-panel);
        background: var(--mc-surface);
        box-shadow: 0 16px 32px rgba(18, 22, 33, 0.06);
    }

    .section-label {
        margin: 0;
        color: var(--mc-primary);
        font-size: 0.78rem;
        letter-spacing: 0.16em;
        text-transform: uppercase;
    }

    h2,
    p {
        margin: 0;
        color: var(--mc-text);
    }

    p:last-child {
        color: var(--mc-text-secondary);
        line-height: 1.6;
    }
</style>
