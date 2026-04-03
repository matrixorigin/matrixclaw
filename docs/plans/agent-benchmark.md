# Agent Benchmark: MatrixClaw vs Hermes Agent

**Date**: 2026-04-03
**Status**: Design Document — No Code
**Purpose**: Define a repeatable scoring system to measure MatrixClaw's progress against Hermes Agent across capability, runtime, and extensibility dimensions.

---

## 1. Capability Scorecard

### Scoring Scale

Each sub-dimension is scored 0–5:

| Score | Meaning |
|-------|---------|
| 0     | Not present / stub only |
| 1     | Basic implementation, major gaps |
| 2     | Functional but limited edge cases |
| 3     | Solid, handles most cases |
| 4     | Comprehensive, minor gaps |
| 5     | Best-in-class, production-ready |

### 1.1 Tool Coverage (weight: 25%)

**Working tool count** — total tools that execute real logic (not stubs).

| Sub-dimension | MatrixClaw | Hermes |
|---------------|-----------|--------|
| Working tool count (not stubs) | ? | ? |
| Filesystem tools (read/write/edit/list/search) | ? | ? |
| Terminal execution | ? | ? |
| Web (fetch + search) | ? | ? |
| Memory (persistent + searchable) | ? | ? |
| Code execution (sandboxed) | ? | ? |
| Agent orchestration (delegate/subagent) | ? | ? |
| Process management | ? | ? |
| Scheduling (cron) | ? | ? |
| Browser automation | ? | ? |
| Parameter completeness per tool | ? | ? |
| Error handling quality per tool | ? | ? |

**Scoring rule**: Each tool category scores 0–5. Working tool count maps to a 0–5 scale: 0–4 tools=1, 5–9=2, 10–14=3, 15–19=4, 20+=5. Average all sub-scores for the dimension total.

### 1.2 Agent Intelligence (weight: 25%)

Evaluated via the 10 benchmark tasks in Section 2, plus additional criteria:

| Sub-dimension | Description |
|---------------|-------------|
| Multi-step completion rate | % of tasks 1–10 that complete successfully |
| Tool selection accuracy | Does the agent pick the right tool for the job? |
| Error recovery | On tool failure: retry, adapt strategy, or give up? |
| Context retention | Does the agent remember earlier turns in a session? |
| Subagent delegation | Does it delegate effectively vs doing everything itself? |
| Task planning | Does it break complex tasks into steps before acting? |

**Scoring rule**: Completion rate is the average score across the 10 tasks (Section 2). Other sub-dimensions are scored manually 0–5 by the evaluator after running all tasks. Average all sub-scores.

### 1.3 Runtime Quality (weight: 20%)

| Sub-dimension | Measurement Method |
|---------------|--------------------|
| Cold start time | `time <agent> chat --message "hello"` from binary |
| Memory usage (idle) | RSS after startup, no active session |
| Memory usage (active) | RSS after 50-turn conversation |
| Binary size / install footprint | `ls -la` on binary vs `du -sh` on Python venv |
| Deployment complexity | Count of external dependencies required |
| Concurrency model | Async runtime capabilities (GIL vs no-GIL) |

**Scoring rule**: Each sub-dimension 0–5. Cold start: <100ms=5, <500ms=4, <1s=3, <3s=2, <5s=1, >5s=0. Binary size: <10MB=5, <30MB=4, <50MB=3, <100MB=2, <200MB=1, >200MB=0. Deployment: single binary=5, binary+1 dep=4, binary+2 deps=3, Python venv=2, Python+Node=1, Docker required=0.

### 1.4 Provider & Cost (weight: 15%)

| Sub-dimension | Description |
|---------------|-------------|
| Multi-provider support | Number of LLM providers with native adapters |
| Fallback chain reliability | Automatic failover on provider failure |
| Cost tracking accuracy | Per-session, per-model cost accumulation |
| Rate limiting | Per-provider request throttling |
| Prompt caching | Token reuse across turns (Anthropic-style) |
| Token counting | Input/output tracking from API responses |

**Scoring rule**: Each sub-dimension 0–5. Multi-provider: 1 provider=1, 2–3=3, 4–5=4, 6+=5. Fallback: none=0, manual=2, automatic=4, automatic+health-checks=5.

### 1.5 Extensibility (weight: 15%)

| Sub-dimension | Description |
|---------------|-------------|
| MCP client | Can consume external MCP tool servers |
| MCP server | Can expose tools to external MCP clients |
| Custom tool addition | Difficulty of adding a new tool (LOC, files touched) |
| Plugin system | Lifecycle hooks, dynamic loading |
| Configuration flexibility | Config file structure, environment variable support |
| Skill/agent customization | Custom agent profiles, prompt templates |

**Scoring rule**: Each sub-dimension 0–5. MCP client: none=0, basic=3, full=5. MCP server: none=0, implemented=3, production=5. Custom tool: >5 files=1, 3–4 files=2, 2 files=3, 1 file=5.

---

## 2. Benchmark Tasks

Each task is run 3 times per agent. Scored 0–5 per run. See Section 3 for scoring criteria.

### Task 1: File Search

**Input prompt:**
```
Find all TODO comments in the src/ directory. List each one with its file path and line number.
```

**Expected behavior:**
- Uses a content search tool (ripgrep-based or equivalent)
- Returns file paths with line numbers
- Does not miss TODOs in subdirectories

**Scoring criteria:**
| Score | Criteria |
|-------|----------|
| 0     | No search performed, or completely wrong |
| 1     | Searches but uses wrong pattern or misses most results |
| 2     | Finds some TODOs but misses subdirectories or edge cases |
| 3     | Finds most TODOs with correct file/line info |
| 4     | Finds all TODOs, correct file/line, minor formatting issues |
| 5     | Finds all TODOs, clean formatted output with count |

### Task 2: Multi-File Edit

**Input prompt:**
```
Rename the function `getUser` to `fetchUser` across all .ts files in this project. Make sure to update both definitions and all call sites.
```

**Setup:** A small TypeScript project with `getUser` in 5+ files.

**Expected behavior:**
- Finds all occurrences across files
- Uses search to locate references
- Edits each file correctly
- Does not rename partial matches (e.g., `getUserId`)

**Scoring criteria:**
| Score | Criteria |
|-------|----------|
| 0     | Does not attempt any edits |
| 1     | Edits one file only, or renames partial matches |
| 2     | Edits most files but misses some call sites |
| 3     | Edits all files correctly but takes excessive turns |
| 4     | Edits all files correctly in reasonable turns |
| 5     | Edits all files correctly, no false positives, efficient |

### Task 3: Code Generation

**Input prompt:**
```
Create a REST API endpoint for user registration with:
- Email validation (must be unique)
- Password strength check (8+ chars, mixed case, number, special)
- Input sanitization
- Proper error responses
Use Express.js and write it to src/routes/register.ts.
```

**Expected behavior:**
- Creates the file with valid TypeScript
- Includes all requested validation
- Has proper error handling
- Code is syntactically correct

**Scoring criteria:**
| Score | Criteria |
|-------|----------|
| 0     | No code generated |
| 1     | Code generated but syntactically invalid |
| 2     | Syntactically valid but missing 2+ requirements |
| 3     | Most requirements met, minor issues |
| 4     | All requirements met, clean code |
| 5     | All requirements, clean code, edge cases handled |

### Task 4: Debugging

**Input prompt:**
```
The tests in this project are failing. Find and fix the bug.
```

**Setup:** A small project with a known bug (e.g., off-by-one error in a sort function, wrong comparison operator, missing null check).

**Expected behavior:**
- Reads test output to identify failures
- Reads relevant source files
- Identifies the bug
- Applies the fix
- Verifies tests pass

**Scoring criteria:**
| Score | Criteria |
|-------|----------|
| 0     | Does not attempt to debug |
| 1     | Reads tests but cannot identify the bug |
| 2     | Identifies the bug but fix is wrong |
| 3     | Identifies and fixes the bug, does not verify |
| 4     | Fixes and verifies with test run |
| 5     | Fixes, verifies, explains root cause clearly |

### Task 5: Research

**Input prompt:**
```
What are the key differences between Rust's async model and Go's goroutines? Focus on:
1. Concurrency model
2. Memory overhead
3. Scheduling
4. Error handling
5. Ecosystem maturity
Provide a structured comparison.
```

**Expected behavior:**
- Uses web search or fetch to gather information (if available)
- Produces a structured comparison
- Information is accurate
- Covers all 5 requested dimensions

**Scoring criteria:**
| Score | Criteria |
|-------|----------|
| 0     | No response or completely fabricated |
| 1     | Response covers <2 dimensions, significant errors |
| 2     | Covers 2–3 dimensions, some errors |
| 3     | Covers most dimensions, mostly accurate |
| 4     | Covers all dimensions, accurate, structured |
| 5     | Covers all dimensions, accurate, well-structured, cites sources |

### Task 6: Memory

**Input prompt (turn 1):**
```
Remember that I prefer tabs over spaces for indentation, and I use 2-space tab width.
```

**Input prompt (turn 2, after clearing conversation context):**
```
What's my indentation preference?
```

**Expected behavior:**
- Stores the preference using the memory tool
- On turn 2, retrieves from persistent memory (not conversation context)
- Returns correct preference

**Scoring criteria:**
| Score | Criteria |
|-------|----------|
| 0     | No memory tool usage |
| 1     | Uses memory but fails to store correctly |
| 2     | Stores correctly but cannot retrieve |
| 3     | Stores and retrieves, partial information |
| 4     | Stores and retrieves all information correctly |
| 5     | Stores, retrieves, and proactively applies the preference |

### Task 7: Multi-Step

**Input prompt:**
```
Set up a new Rust project called "weather-cli" in the current directory with:
1. cargo init
2. A src/main.rs that prints "Hello, weather!"
3. Unit tests in src/lib.rs
4. A GitHub Actions CI config at .github/workflows/ci.yml
5. A README.md with project description and build instructions
```

**Expected behavior:**
- Plans the steps (ideally using todo/task tool)
- Executes each step in order
- Creates all files with correct content
- Verifies the project compiles

**Scoring criteria:**
| Score | Criteria |
|-------|----------|
| 0     | Does not attempt |
| 1     | Completes 1–2 of the 5 steps |
| 2     | Completes 3 of the 5 steps |
| 3     | Completes 4 of the 5 steps |
| 4     | Completes all 5 steps |
| 5     | All 5 steps + verifies compilation + clean commit |

### Task 8: Subagent / Parallel

**Input prompt:**
```
Analyze these 3 code files in parallel and give me a summary of each:
1. src/auth.ts — security assessment
2. src/api.ts — API design review
3. src/db.ts — data model review
```

**Expected behavior:**
- Uses delegate/subagent tool to spawn parallel analyses
- Each analysis is substantive (not trivial)
- Results are aggregated into a summary

**Scoring criteria:**
| Score | Criteria |
|-------|----------|
| 0     | No subagent usage |
| 1     | Processes sequentially instead of parallel |
| 2     | Attempts parallel but one fails |
| 3     | Parallel execution works, shallow analysis |
| 4     | Parallel execution, substantive analysis |
| 5     | Parallel execution, substantive, well-aggregated summary |

### Task 9: Scheduled Task

**Input prompt:**
```
Set up a daily task that runs every 24 hours and generates a git status report for the current repository. Save the report to reports/daily-status.txt.
```

**Expected behavior:**
- Uses cron/scheduling tool to create the job
- Job has correct interval
- Task prompt is reasonable
- Job appears in job list

**Scoring criteria:**
| Score | Criteria |
|-------|----------|
| 0     | No scheduling capability |
| 1     | Creates a one-off script instead of scheduled task |
| 2     | Uses scheduling tool but wrong interval |
| 3     | Correct interval, task prompt is vague |
| 4     | Correct interval, clear task prompt, job listed |
| 5     | All of 4 + can verify the job will execute correctly |

### Task 10: Error Recovery

**Input prompt:**
```
Deploy this application by running: ./deploy.sh
```

**Setup:** A `deploy.sh` script that fails with a specific error (e.g., missing environment variable, port already in use, missing dependency).

**Expected behavior:**
- Runs the script
- Reads the error output
- Diagnoses the issue
- Fixes the issue (set env var, kill process, install dep)
- Re-runs the deploy
- Reports success

**Scoring criteria:**
| Score | Criteria |
|-------|----------|
| 0     | Runs script, sees error, gives up |
| 1     | Identifies error type but wrong fix |
| 2     | Identifies and applies fix, does not re-run |
| 3     | Fixes and re-runs, but deploy still fails |
| 4     | Fixes, re-runs, deploy succeeds |
| 5     | Fixes, re-runs, succeeds, explains what was wrong |

---

## 3. Scoring Method

### Execution Protocol

1. **Environment**: Run both agents on the same machine, same LLM model, same API provider
2. **Model control**: Use the same model for both agents (e.g., `anthropic/claude-sonnet-4` via OpenRouter)
3. **Runs per task**: 3 runs per task per agent (9 runs total for 3 agents, or 6 for 2)
4. **Isolation**: Fresh session per run, no carry-over state (except Task 6 which tests persistence)
5. **Timeout**: 5 minutes per task. Agent gets 0 if it exceeds timeout
6. **Evaluation**: Score each run independently on the 0–5 rubric

### Score Calculation

```
task_score[agent][task] = mean(runs[agent][task][0..2])

dimension_score[agent]["tool_coverage"]    = mean(sub_scores for tool coverage)
dimension_score[agent]["intelligence"]     = mean(task_scores[agent][1..10])
dimension_score[agent]["runtime"]          = mean(sub_scores for runtime)
dimension_score[agent]["provider_cost"]    = mean(sub_scores for provider & cost)
dimension_score[agent]["extensibility"]    = mean(sub_scores for extensibility)

weights = {
    "tool_coverage":  0.25,
    "intelligence":   0.25,
    "runtime":        0.20,
    "provider_cost":  0.15,
    "extensibility":  0.15,
}

total_score[agent] = sum(dimension_score[agent][d] * weights[d] for d in dimensions)
```

Maximum possible score: **5.00**

### Result Presentation

Results are presented as:

```
┌──────────────────────┬────────────┬────────────┐
│ Dimension            │ MatrixClaw │ Hermes     │
├──────────────────────┼────────────┼────────────┤
│ Tool Coverage (25%)  │ x.xx / 5   │ x.xx / 5   │
│ Intelligence (25%)   │ x.xx / 5   │ x.xx / 5   │
│ Runtime (20%)        │ x.xx / 5   │ x.xx / 5   │
│ Provider & Cost(15%) │ x.xx / 5   │ x.xx / 5   │
│ Extensibility (15%)  │ x.xx / 5   │ x.xx / 5   │
├──────────────────────┼────────────┼────────────┤
│ WEIGHTED TOTAL       │ x.xx / 5   │ x.xx / 5   │
└──────────────────────┴────────────┴────────────┘
```

Plus a per-task breakdown table.

---

## 4. Current Score Estimates

Based on analysis of both codebases as of 2026-04-03. MatrixClaw is at Phase 6/7 complete (web_search, code_interpreter, MCP server, prompt caching done; browser automation in-progress). Hermes is a mature Python agent with 23k+ GitHub stars.

### 4.1 Tool Coverage Estimates

| Sub-dimension | MatrixClaw | Score | Hermes | Score |
|---------------|-----------|-------|--------|-------|
| Working tool count | 18+ tools (no stubs) | 4 | 40+ tools | 5 |
| Filesystem | read/write/edit/list/search — all full | 5 | read/write/edit/list/search — all full | 5 |
| Terminal | Full, with timeout | 4 | Full, multiple backends | 5 |
| Web | fetch+search (Exa/Tavily) — real search | 4 | fetch+search (Exa/Tavily) | 5 |
| Memory | SQLite-backed, persistent, searchable | 4 | Persistent, searchable | 4 |
| Code execution | Full (Docker sandbox) | 4 | Full (Docker sandbox) | 5 |
| Agent orchestration | delegate tool + MCP server, depth-2, callback architecture | 4 | Full subagent system | 4 |
| Process management | Full (list/register/kill) | 4 | Full | 4 |
| Scheduling | Cron tool, SQLite-backed, interval+natural language | 4 | Full cron, natural language | 4 |
| Browser automation | Placeholder (Phase 7.3 in-progress) | 0 | 11 browser tools | 5 |
| Parameter quality | Consistent JSON schema, well-typed | 4 | Comprehensive | 5 |
| Error handling | Descriptive errors, command approval system | 4 | Mature, retry mechanisms | 5 |

**Dimension average — Tool Coverage:**
- **MatrixClaw: 3.75 / 5**
- **Hermes: 4.7 / 5**

### 4.2 Agent Intelligence Estimates

Based on tool availability and loop maturity:

| Task | MatrixClaw Est. | Rationale | Hermes Est. | Rationale |
|------|----------------|-----------|-------------|-----------|
| T1: File search | 4 | search_files tool works well | 5 | Mature ripgrep integration |
| T2: Multi-file edit | 4 | edit_file works, command approval for safety | 4 | 9-strategy patch tool |
| T3: Code generation | 4 | Can write + verify with code_interpreter | 4 | Auto-runs linter/tests |
| T4: Debugging | 4 | Can read/test/edit in sandbox | 4 | Proven debug workflows |
| T5: Research | 4 | Real web search (Exa/Tavily) | 4 | Real search + browser tools |
| T6: Memory | 4 | SQLite-backed memory tool | 4 | Persistent memory |
| T7: Multi-step | 3 | Has todo tool, no context compression tuning | 4 | Proven multi-step workflows |
| T8: Subagent | 3 | delegate works, no parallel execution yet | 4 | Parallel subagent execution |
| T9: Scheduled task | 3 | cronjob tool works, interval-based | 4 | Full cron with natural language |
| T10: Error recovery | 3 | Command approval + sandbox for recovery | 4 | Proven error recovery patterns |

**Dimension average — Intelligence:**
- **MatrixClaw: 3.6 / 5**
- **Hermes: 4.1 / 5**

### 4.3 Runtime Quality Estimates

| Sub-dimension | MatrixClaw | Score | Hermes | Score |
|---------------|-----------|-------|--------|-------|
| Cold start time | <100ms (native binary) | 5 | 2–5s (Python + imports) | 2 |
| Memory usage (idle) | ~5–15MB RSS | 5 | ~80–150MB RSS | 3 |
| Memory usage (active) | ~20–50MB RSS | 5 | ~200–500MB RSS | 2 |
| Binary size | ~8–15MB | 4 | ~500MB+ (Python venv + deps) | 1 |
| Deployment complexity | Single binary, zero deps | 5 | Python + Node.js + pip install | 1 |
| Concurrency | Rust async, no GIL | 5 | Python GIL (async with limitations) | 2 |

**Dimension average — Runtime Quality:**
- **MatrixClaw: 4.8 / 5**
- **Hermes: 1.8 / 5**

### 4.4 Provider & Cost Estimates

| Sub-dimension | MatrixClaw | Score | Hermes | Score |
|---------------|-----------|-------|--------|-------|
| Multi-provider | OpenAI-compatible (works with any /v1 endpoint) | 3 | OpenAI, Anthropic, Google, local | 4 |
| Fallback chains | Full, with health checks + rate limiting | 5 | Manual provider switching | 2 |
| Cost tracking | SQLite-backed, per-session, per-model | 5 | Basic token counting | 2 |
| Rate limiting | Token-bucket per provider | 5 | Not built-in | 1 |
| Prompt caching | system_and_3 strategy, auto-enabled for Claude | 5 | Anthropic system_and_3 | 4 |
| Token counting | From API usage fields | 4 | From API usage fields | 4 |

**Dimension average — Provider & Cost:**
- **MatrixClaw: 4.5 / 5**
- **Hermes: 2.8 / 5**

### 4.5 Extensibility Estimates

| Sub-dimension | MatrixClaw | Score | Hermes | Score |
|---------------|-----------|-------|--------|-------|
| MCP client | Full (JSON-RPC over stdio) | 4 | Full | 4 |
| MCP server | Full (`matrixclaw mcp-serve`) | 4 | Full (`hermes mcp serve`) | 5 |
| Custom tool difficulty | ~1 file, implement ToolExecutor trait | 4 | ~1 file, Python class | 4 |
| Plugin system | Lifecycle hooks (6 hook points via MCP) | 3 | 4 lifecycle hooks | 4 |
| Config flexibility | JSON configs + env vars | 3 | YAML + env vars + CLI flags | 4 |
| Skill/agent customization | Skills tool, manifests | 3 | Skills + self-evolving (DSPy) | 5 |

**Dimension average — Extensibility:**
- **MatrixClaw: 3.5 / 5**
- **Hermes: 4.3 / 5**

### 4.6 Weighted Totals

| Dimension | Weight | MatrixClaw | Weighted | Hermes | Weighted |
|-----------|--------|-----------|----------|--------|----------|
| Tool Coverage | 25% | 3.75 | 0.94 | 4.7 | 1.18 |
| Intelligence | 25% | 3.6 | 0.90 | 4.1 | 1.03 |
| Runtime Quality | 20% | 4.8 | 0.96 | 1.8 | 0.36 |
| Provider & Cost | 15% | 4.5 | 0.68 | 2.8 | 0.42 |
| Extensibility | 15% | 3.5 | 0.53 | 4.3 | 0.65 |
| **TOTAL** | **100%** | | **4.00** | | **3.63** |

### 4.7 Gap Analysis Summary

**MatrixClaw leads in:**
- Runtime quality (+3.0) — the single-binary Rust advantage is massive
- Provider infrastructure (+1.7) — fallback chains, cost tracking, rate limiting, prompt caching
- These are structural advantages that Hermes cannot easily replicate

**MatrixClaw now overtakes Hermes overall (4.00 vs 3.63).**

**Hermes leads in:**
- Tool coverage (+0.95) — especially browser automation (11 tools)
- Agent intelligence (+0.5) — proven workflows, parallel subagents
- Extensibility (+0.8) — self-evolving skills, more mature plugin hooks

**Remaining gaps to close (priority order):**
1. Browser automation (biggest remaining tool gap — Phase 7.3 in-progress)
2. Parallel subagent execution (intelligence gap)
3. Context compression tuning (intelligence gap)
4. Fuzzy/9-strategy file editing (intelligence gap for multi-file edits)

**If MatrixClaw closes browser automation + parallel subagents, the estimated total rises to ~4.3+.**

---

## 5. Benchmark CLI Design

### Command Interface

```
matrixclaw benchmark run [OPTIONS]

Options:
  --tasks <TASKS>      Task IDs to run (comma-separated) or "all" [default: all]
  --runs <N>           Number of runs per task [default: 3]
  --model <MODEL>      LLM model to use [default: from config]
  --output <PATH>      Output file path [default: benchmark-results.json]
  --compare <PATH>     Compare against previous results file
  --timeout <SECS>     Per-task timeout in seconds [default: 300]
  --agent <AGENT>      Agent to benchmark: "matrixclaw" or "hermes" [default: matrixclaw]
```

### Data Structure

```json
{
  "version": "1.0",
  "timestamp": "2026-04-03T12:00:00Z",
  "config": {
    "model": "anthropic/claude-sonnet-4",
    "runs_per_task": 3,
    "timeout_seconds": 300,
    "agent": "matrixclaw",
    "agent_version": "0.5.0"
  },
  "tasks": [
    {
      "id": "t1_file_search",
      "name": "File Search",
      "prompt": "Find all TODO comments in the src/ directory...",
      "runs": [
        {
          "run_number": 1,
          "score": 4,
          "scorer_notes": "Found all TODOs, clean output",
          "duration_seconds": 12.3,
          "tool_calls": 2,
          "tokens_used": {"input": 1500, "output": 450},
          "cost_usd": 0.0023,
          "error": null
        },
        {
          "run_number": 2,
          "score": 3,
          "scorer_notes": "Missed TODOs in nested src/utils/",
          "duration_seconds": 15.1,
          "tool_calls": 3,
          "tokens_used": {"input": 1800, "output": 520},
          "cost_usd": 0.0029,
          "error": null
        }
      ],
      "average_score": 3.7
    }
  ],
  "dimensions": {
    "tool_coverage": {
      "weight": 0.25,
      "sub_scores": {
        "working_tool_count": 3,
        "filesystem": 5,
        "terminal": 4
      },
      "average": 3.0
    },
    "intelligence": {
      "weight": 0.25,
      "task_scores": {
        "t1_file_search": 3.7,
        "t2_multi_file_edit": 3.0
      },
      "average": 2.9
    },
    "runtime": {
      "weight": 0.20,
      "sub_scores": {
        "cold_start_ms": 85,
        "idle_memory_mb": 12,
        "binary_size_mb": 11
      },
      "average": 4.8
    },
    "provider_cost": {
      "weight": 0.15,
      "average": 3.7
    },
    "extensibility": {
      "weight": 0.15,
      "average": 2.5
    }
  },
  "weighted_total": 4.00
}
```

### Compare Mode

When `--compare` is passed with a previous results file:

```
matrixclaw benchmark run --tasks all --runs 3 --compare baseline-2026-03.json

Output:
  DIMENSION             THIS RUN    BASELINE    DELTA
  Tool Coverage (25%)   3.0 / 5     2.5 / 5     +0.5
  Intelligence (25%)    2.9 / 5     2.6 / 5     +0.3
  Runtime (20%)         4.8 / 5     4.8 / 5      0.0
  Provider & Cost(15%)  3.7 / 5     3.7 / 5      0.0
  Extensibility (15%)   2.5 / 5     2.0 / 5     +0.5
  ──────────────────────────────────────────────────
  TOTAL                 3.38 / 5    3.08 / 5    +0.30
```

### Implementation Notes

- Task scoring requires a human evaluator for now — the CLI captures raw output and timing, the evaluator fills in scores
- Future: LLM-as-judge for automated scoring (score the agent's output with a stronger model)
- Runtime metrics (cold start, memory, binary size) are fully automatable
- Provider metrics (token counts, cost) are extracted from the provider plane automatically
- The `--agent hermes` mode would shell out to the Hermes CLI with the same prompts and capture output

---

## Appendix A: Task Setup Requirements

Each task requires specific test fixtures:

| Task | Setup Required |
|------|---------------|
| T1 | A repo with 20+ TODO comments across nested directories |
| T2 | A TypeScript project with `getUser` in 5+ files |
| T3 | An Express.js project scaffold (package.json exists) |
| T4 | A project with a known, reproducible bug and failing tests |
| T5 | No setup (knowledge task) |
| T6 | Fresh agent instance, two-turn conversation |
| T7 | Empty directory with Rust/cargo available |
| T8 | A project with 3 substantial code files |
| T9 | A git repository |
| T10 | A project with a deploy.sh that fails predictably |

## Appendix B: Version Pinning

To ensure reproducibility:

```
MatrixClaw: git sha or version tag
Hermes: git sha or version tag
LLM Model: exact model string (e.g., "anthropic/claude-sonnet-4-20250514")
Provider: OpenRouter (or specific provider)
Date: ISO 8601
OS: macOS / Linux (specify version)
```
