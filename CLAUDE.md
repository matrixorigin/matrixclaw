## Architecture

Read `docs/plans/runtime-rethink.md` for the full roadmap.
Read `DESIGN.md` for runtime architecture.

## Validation

```bash
cargo check --workspace                    # compile check
cargo test --workspace                     # all tests
cargo clippy --workspace --all-targets     # lint
cargo fmt --all -- --check                 # format check
cargo build --release                      # release build
```

## LLM Smoke Test

Requires an API key. Set `OPENROUTER_API_KEY` and optionally `MATRIXCLAW_LLM_MODEL`:

```bash
OPENROUTER_API_KEY=sk-or-... cargo run -p matrixclaw-app-host --bin matrixclaw llm-smoke --model moonshotai/kimi-k2.5
```
