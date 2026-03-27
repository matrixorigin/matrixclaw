<script lang="ts">
    import { executionPriority, setupSteps } from "$lib/shell";
</script>

<section class="landing">
    <div class="hero-card">
        <p class="tag">Browser-first shell</p>
        <h2>Developer landing surface for embedded UI builds</h2>
        <p>
            MatrixClaw will serve static UI assets from the Rust runtime. This root page exists so
            preview builds have a concrete entry point while `app-host` decides whether to send a
            user to setup or workspace at runtime.
        </p>
    </div>

    <div class="grid">
        <article>
            <h3>Setup flow</h3>
            <ol>
                {#each setupSteps as step}
                    <li>{step}</li>
                {/each}
            </ol>
            <a href="/setup">Open setup scaffold</a>
        </article>

        <article>
            <h3>Workspace shell</h3>
            <p>Three regions are pre-shaped for files, transcript/composer, and runtime detail.</p>
            <a href="/workspace">Open workspace scaffold</a>
        </article>

        <article>
            <h3>Sandbox priority</h3>
            <p>
                Sandboxed execution is designed to prefer <strong>{executionPriority[0]}</strong>
                and fall back to <strong>{executionPriority[1]}</strong> only when policy allows it.
            </p>
        </article>
    </div>
</section>

<style>
    .landing {
        display: grid;
        gap: 1.25rem;
    }

    .hero-card,
    article {
        padding: 1.35rem;
        border: 1px solid rgba(148, 163, 184, 0.18);
        border-radius: 1.25rem;
        background: rgba(15, 23, 42, 0.65);
        box-shadow: 0 16px 42px rgba(2, 6, 23, 0.24);
    }

    .tag {
        margin: 0 0 0.5rem;
        color: #fca5a5;
        text-transform: uppercase;
        letter-spacing: 0.16em;
        font-size: 0.78rem;
    }

    h2,
    h3 {
        margin: 0 0 0.6rem;
    }

    p,
    li {
        color: #cbd5e1;
        line-height: 1.6;
    }

    .grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr));
        gap: 1rem;
    }

    a {
        display: inline-flex;
        margin-top: 0.85rem;
        color: #fde68a;
        font-weight: 600;
    }
</style>
