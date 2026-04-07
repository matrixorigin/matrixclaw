# Self-Evolving Skills (GePA + MiProv2 in Rust) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust-native skill optimization loop inspired by DSPy's GePA (Generalized Prompt Adaptation) and MiProv2 (Mining Prompts v2) — without any Python dependency. Skills automatically improve from execution feedback by accumulating success/failure examples, detecting failure patterns, and rewriting their own instructions.

**Architecture:** A new `SkillEvolver` subsystem lives in `crates/matrixclaw-tools/src/builtin/` alongside the existing `SkillsTool`. It has three layers: (1) **TraceCollector** — a `LifecycleHook` that observes every `skills.read` call and records whether the subsequent tool chain succeeded or failed, storing traces in SQLite. (2) **TraceAnalyzer** — groups traces by skill, computes per-skill success rates, and identifies failure patterns (common error sequences). (3) **SkillRewriter** — when a skill's success rate drops below a threshold or N new failures accumulate, it constructs a rewrite prompt containing the current skill + failure examples + success examples, calls the LLM via `Provider`, and writes the improved version as a new skill version with rollback support.

**Tech Stack:** `rusqlite` (trace storage, already used), `serde_json` (trace serialization), `matrixclaw-agent-core::Provider` (LLM calls for rewriting), existing `LifecycleHook` infrastructure, existing `SkillsTool`.

---

## Key Design Decisions

### Why not DSPy directly?
DSPy requires Python + PyTorch. We're a single-binary Rust agent. The two valuable ideas from DSPy are:
- **GePA**: Generalized prompt adaptation — adapt skill instructions based on concrete execution feedback, not abstract optimization
- **MiProv2**: Mine effective prompts from execution traces — look at which traces succeeded vs failed, identify differentiating factors, generate improved prompts

Both reduce to: **collect traces → analyze patterns → rewrite instructions using the LLM itself**. No gradient descent needed.

### Circular dependency constraint
`matrixclaw-tools` cannot depend on `agent-core` (circular). The `Provider` trait lives in `agent-core`. Solution: the `SkillRewriter` takes a callback `Box<dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<String, String>>>>` (similar to the `SubagentRunner` pattern in `delegate.rs`). The callback is wired in `app-host` where both crates are available.

### Where traces live
New SQLite database at `~/.matrixclaw/state/skill_traces.sqlite3`, separate from `memory.sqlite3`. Keeps concerns isolated. Uses the same `Mutex<Connection>` pattern as `MemoryTool`.

---

## File Structure

| File | Responsibility |
|------|---------------|
| `crates/matrixclaw-tools/src/builtin/skill_trace.rs` | `TraceCollector` (LifecycleHook), `SkillTrace`, SQLite storage |
| `crates/matrixclaw-tools/src/builtin/skill_evolver.rs` | `TraceAnalyzer` + `SkillRewriter` + `SkillVersion` |
| `crates/matrixclaw-tools/src/builtin/skills.rs` | Modified: add `history` action for version listing, version-aware `read` |
| `crates/matrixclaw-tools/src/builtin/mod.rs` | Modified: register new modules + `skill_evolve` tool |
| `crates/matrixclaw-tools/src/lib.rs` | Modified: expose `SkillEvolver` types if needed |
| `crates/agent-core/src/lib.rs` | No changes needed (callback pattern avoids circular dep) |
| `crates/app-host/src/live_runtime.rs` | Modified: wire `TraceCollector` into hooks, wire `SkillRewriter` callback |
| `crates/app-host/src/chat.rs` | Modified: register `skill_evolve` tool |

---

## Task 1: SkillTrace Types and SQLite Storage

**Files:**
- Create: `crates/matrixclaw-tools/src/builtin/skill_trace.rs`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write the failing tests**

```rust
// In crates/matrixclaw-tools/src/builtin/skill_trace.rs

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTrace {
    pub id: i64,
    pub skill_name: String,
    pub task_summary: String,
    pub outcome: TraceOutcome,
    pub tool_chain: Vec<ToolInvocation>,
    pub llm_output_snippet: String,
    pub iteration: u32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TraceOutcome {
    Success,
    Failure,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    pub tool: String,
    pub arguments_summary: String,
    pub result_summary: String,
}

pub struct TraceStore {
    db: Mutex<Connection>,
}

impl TraceStore {
    pub fn open(db_path: &Path) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("trace: failed to create db directory: {e}"))?;
        }
        let conn =
            Connection::open(db_path).map_err(|e| format!("trace: failed to open db: {e}"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS skill_traces (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                skill_name TEXT NOT NULL,
                task_summary TEXT NOT NULL,
                outcome TEXT NOT NULL,
                tool_chain TEXT NOT NULL,
                llm_output_snippet TEXT NOT NULL,
                iteration INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_traces_skill ON skill_traces(skill_name);
            CREATE INDEX IF NOT EXISTS idx_traces_outcome ON skill_traces(outcome);
            CREATE VIRTUAL TABLE IF NOT EXISTS traces_fts USING fts5(
                task_summary,
                content=skill_traces,
                content_rowid=id
            );
            CREATE TRIGGER IF NOT EXISTS traces_ai AFTER INSERT ON skill_traces BEGIN
                INSERT INTO traces_fts(rowid, task_summary) VALUES (new.id, new.task_summary);
            END;
            CREATE TRIGGER IF NOT EXISTS traces_ad AFTER DELETE ON skill_traces BEGIN
                INSERT INTO traces_fts(traces_fts, rowid, task_summary) VALUES('delete', old.id, old.task_summary);
            END;",
        )
        .map_err(|e| format!("trace: failed to create tables: {e}"))?;
        Ok(Self {
            db: Mutex::new(conn),
        })
    }

    pub fn db_path_for_home(home: &Path) -> std::path::PathBuf {
        home.join(".matrixclaw")
            .join("state")
            .join("skill_traces.sqlite3")
    }

    pub fn insert(&self, trace: &SkillTrace) -> Result<i64, String> {
        let db = self.db.lock().map_err(|e| format!("lock: {e}"))?;
        let tool_chain_json = serde_json::to_string(&trace.tool_chain)
            .map_err(|e| format!("serialize: {e}"))?;
        db.execute(
            "INSERT INTO skill_traces (skill_name, task_summary, outcome, tool_chain, llm_output_snippet, iteration, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                trace.skill_name,
                trace.task_summary,
                serde_json::to_string(&trace.outcome).unwrap(),
                tool_chain_json,
                trace.llm_output_snippet,
                trace.iteration,
                trace.created_at,
            ],
        )
        .map_err(|e| format!("insert: {e}"))?;
        Ok(db.last_insert_rowid())
    }

    pub fn get_traces_for_skill(&self, skill_name: &str, limit: usize) -> Result<Vec<SkillTrace>, String> {
        let db = self.db.lock().map_err(|e| format!("lock: {e}"))?;
        let mut stmt = db
            .prepare("SELECT id, skill_name, task_summary, outcome, tool_chain, llm_output_snippet, iteration, created_at FROM skill_traces WHERE skill_name = ?1 ORDER BY created_at DESC LIMIT ?2")
            .map_err(|e| format!("prepare: {e}"))?;
        let rows = stmt
            .query_map(params![skill_name, limit], |row| {
                let id: i64 = row.get(0)?;
                let skill_name: String = row.get(1)?;
                let task_summary: String = row.get(2)?;
                let outcome_str: String = row.get(3)?;
                let tool_chain_str: String = row.get(4)?;
                let llm_output_snippet: String = row.get(5)?;
                let iteration: u32 = row.get(6)?;
                let created_at: String = row.get(7)?;
                Ok((
                    id,
                    skill_name,
                    task_summary,
                    outcome_str,
                    tool_chain_str,
                    llm_output_snippet,
                    iteration,
                    created_at,
                ))
            })
            .map_err(|e| format!("query: {e}"))?;
        let mut traces = Vec::new();
        for row in rows {
            let (id, skill_name, task_summary, outcome_str, tool_chain_str, llm_output_snippet, iteration, created_at) =
                row.map_err(|e| format!("row: {e}"))?;
            let outcome: TraceOutcome =
                serde_json::from_str(&outcome_str).unwrap_or(TraceOutcome::Partial);
            let tool_chain: Vec<ToolInvocation> =
                serde_json::from_str(&tool_chain_str).unwrap_or_default();
            traces.push(SkillTrace {
                id,
                skill_name,
                task_summary,
                outcome,
                tool_chain,
                llm_output_snippet,
                iteration,
                created_at,
            });
        }
        Ok(traces)
    }

    pub fn get_success_rate(&self, skill_name: &str, last_n: usize) -> Result<f64, String> {
        let db = self.db.lock().map_err(|e| format!("lock: {e}"))?;
        let total: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM skill_traces WHERE skill_name = ?1 ORDER BY created_at DESC LIMIT ?2",
                params![skill_name, last_n as i64],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if total == 0 {
            return Ok(1.0);
        }
        let successes: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM skill_traces WHERE skill_name = ?1 AND outcome = '\"success\"' ORDER BY created_at DESC LIMIT ?2",
                params![skill_name, last_n as i64],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(successes as f64 / total as f64)
    }

    pub fn count_recent_failures(&self, skill_name: &str, since_hours: i64) -> Result<usize, String> {
        let db = self.db.lock().map_err(|e| format!("lock: {e}"))?;
        let count: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM skill_traces WHERE skill_name = ?1 AND outcome = '\"failure\"' AND created_at >= datetime('now', ?2)",
                params![skill_name, format!("-{since_hours} hours")],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(count as usize)
    }

    pub fn search_traces(&self, query: &str, limit: usize) -> Result<Vec<SkillTrace>, String> {
        let db = self.db.lock().map_err(|e| format!("lock: {e}"))?;
        let mut stmt = db
            .prepare(
                "SELECT t.id, t.skill_name, t.task_summary, t.outcome, t.tool_chain, t.llm_output_snippet, t.iteration, t.created_at FROM skill_traces t JOIN traces_fts f ON t.id = f.rowid WHERE traces_fts MATCH ?1 ORDER BY rank LIMIT ?2",
            )
            .map_err(|e| format!("prepare: {e}"))?;
        let rows = stmt
            .query_map(params![query, limit], |row| {
                let id: i64 = row.get(0)?;
                let skill_name: String = row.get(1)?;
                let task_summary: String = row.get(2)?;
                let outcome_str: String = row.get(3)?;
                let tool_chain_str: String = row.get(4)?;
                let llm_output_snippet: String = row.get(5)?;
                let iteration: u32 = row.get(6)?;
                let created_at: String = row.get(7)?;
                Ok((
                    id, skill_name, task_summary, outcome_str, tool_chain_str,
                    llm_output_snippet, iteration, created_at,
                ))
            })
            .map_err(|e| format!("query: {e}"))?;
        let mut traces = Vec::new();
        for row in rows {
            let (id, skill_name, task_summary, outcome_str, tool_chain_str, llm_output_snippet, iteration, created_at) =
                row.map_err(|e| format!("row: {e}"))?;
            let outcome: TraceOutcome =
                serde_json::from_str(&outcome_str).unwrap_or(TraceOutcome::Partial);
            let tool_chain: Vec<ToolInvocation> =
                serde_json::from_str(&tool_chain_str).unwrap_or_default();
            traces.push(SkillTrace {
                id, skill_name, task_summary, outcome, tool_chain, llm_output_snippet, iteration, created_at,
            });
        }
        Ok(traces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TraceStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let store = TraceStore::open(&dir.path().join("traces.sqlite3")).unwrap();
        (store, dir)
    }

    fn make_trace(skill: &str, task: &str, outcome: TraceOutcome) -> SkillTrace {
        SkillTrace {
            id: 0,
            skill_name: skill.to_string(),
            task_summary: task.to_string(),
            outcome,
            tool_chain: vec![ToolInvocation {
                tool: "terminal".to_string(),
                arguments_summary: "ls -la".to_string(),
                result_summary: "file list".to_string(),
            }],
            llm_output_snippet: "did the thing".to_string(),
            iteration: 1,
            created_at: "2026-04-07 12:00:00".to_string(),
        }
    }

    #[test]
    fn insert_and_retrieve() {
        let (store, _dir) = setup();
        let trace = make_trace("deploy", "deploy to staging", TraceOutcome::Success);
        let id = store.insert(&trace).unwrap();
        assert!(id > 0);
        let traces = store.get_traces_for_skill("deploy", 10).unwrap();
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].skill_name, "deploy");
        assert_eq!(traces[0].outcome, TraceOutcome::Success);
    }

    #[test]
    fn success_rate_calculation() {
        let (store, _dir) = setup();
        store.insert(&make_trace("s1", "t1", TraceOutcome::Success)).unwrap();
        store.insert(&make_trace("s1", "t2", TraceOutcome::Success)).unwrap();
        store.insert(&make_trace("s1", "t3", TraceOutcome::Failure)).unwrap();
        let rate = store.get_success_rate("s1", 10).unwrap();
        assert!((rate - 0.6667).abs() < 0.05);
    }

    #[test]
    fn success_rate_empty_is_one() {
        let (store, _dir) = setup();
        let rate = store.get_success_rate("nope", 10).unwrap();
        assert_eq!(rate, 1.0);
    }

    #[test]
    fn count_recent_failures() {
        let (store, _dir) = setup();
        store.insert(&make_trace("s1", "t1", TraceOutcome::Failure)).unwrap();
        store.insert(&make_trace("s1", "t2", TraceOutcome::Success)).unwrap();
        let count = store.count_recent_failures("s1", 24).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn search_traces_by_task() {
        let (store, _dir) = setup();
        store.insert(&make_trace("deploy", "deploy k8s staging", TraceOutcome::Success)).unwrap();
        store.insert(&make_trace("test", "run unit tests", TraceOutcome::Success)).unwrap();
        let results = store.search_traces("k8s", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].skill_name, "deploy");
    }

    #[test]
    fn tool_chain_roundtrip() {
        let (store, _dir) = setup();
        let trace = SkillTrace {
            id: 0,
            skill_name: "multi".to_string(),
            task_summary: "complex task".to_string(),
            outcome: TraceOutcome::Partial,
            tool_chain: vec![
                ToolInvocation {
                    tool: "read_file".to_string(),
                    arguments_summary: "src/main.rs".to_string(),
                    result_summary: "200 lines".to_string(),
                },
                ToolInvocation {
                    tool: "terminal".to_string(),
                    arguments_summary: "cargo build".to_string(),
                    result_summary: "Compiled successfully".to_string(),
                },
            ],
            llm_output_snippet: "built it".to_string(),
            iteration: 3,
            created_at: "2026-04-07 12:00:00".to_string(),
        };
        store.insert(&trace).unwrap();
        let retrieved = store.get_traces_for_skill("multi", 10).unwrap();
        assert_eq!(retrieved[0].tool_chain.len(), 2);
        assert_eq!(retrieved[0].tool_chain[0].tool, "read_file");
        assert_eq!(retrieved[0].tool_chain[1].tool, "terminal");
    }
}
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test -p matrixclaw-tools skill_trace`
Expected: All 7 tests PASS

- [ ] **Step 3: Register the module**

In `crates/matrixclaw-tools/src/builtin/mod.rs`, add:

```rust
pub mod skill_trace;
```

- [ ] **Step 4: Run full workspace test**

Run: `cargo test --workspace`
Expected: All existing tests + 7 new tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/matrixclaw-tools/src/builtin/skill_trace.rs crates/matrixclaw-tools/src/builtin/mod.rs
git commit -m "feat(tools): add SkillTrace storage with SQLite backend

TraceStore records skill execution traces (success/failure/partial) with
tool chain details, FTS5 search, success rate calculation, and recent
failure counting. Foundation for self-evolving skills."
```

---

## Task 2: TraceCollector Lifecycle Hook

**Files:**
- Modify: `crates/matrixclaw-tools/src/builtin/skill_trace.rs` (add `TraceCollector`)
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write the failing tests**

Add to the `#[cfg(test)]` block in `skill_trace.rs`:

```rust
use async_trait::async_trait;
use matrixclaw_agent_core::hooks::{HookAction, HookPayload, HookPoint, LifecycleHook};

struct TraceCollector {
    store: TraceStore,
    active_skill: Mutex<Option<String>>,
    active_task: Mutex<Option<String>>,
    tool_buffer: Mutex<Vec<ToolInvocation>>,
    iteration_buffer: Mutex<u32>,
}

impl TraceCollector {
    fn new(store: TraceStore) -> Self {
        Self {
            store,
            active_skill: Mutex::new(None),
            active_task: Mutex::new(None),
            tool_buffer: Mutex::new(Vec::new()),
            iteration_buffer: Mutex::new(0),
        }
    }

    fn on_skill_read(&self, skill_name: &str, task_summary: &str) {
        *self.active_skill.lock().unwrap() = Some(skill_name.to_string());
        *self.active_task.lock().unwrap() = Some(task_summary.to_string());
        self.tool_buffer.lock().unwrap().clear();
        *self.iteration_buffer.lock().unwrap() = 0;
    }

    fn finalize(&self, outcome: TraceOutcome, llm_snippet: &str) -> Result<i64, String> {
        let skill_name = self.active_skill.lock().unwrap().take();
        let task_summary = self.active_task.lock().unwrap().take();
        let tool_chain = self.tool_buffer.lock().unwrap().drain(..).collect();
        let iteration = *self.iteration_buffer.lock().unwrap();

        match (skill_name, task_summary) {
            (Some(skill), Some(task)) => {
                let trace = SkillTrace {
                    id: 0,
                    skill_name: skill,
                    task_summary: task,
                    outcome,
                    tool_chain,
                    llm_output_snippet: llm_snippet.chars().take(500).collect(),
                    iteration,
                    created_at: chrono_now(),
                };
                self.store.insert(&trace)
            }
            _ => Ok(0),
        }
    }
}

fn chrono_now() -> String {
    use std::time::SystemTime;
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("1970-01-01 00:00:{secs:02}")
}

#[async_trait]
impl LifecycleHook for TraceCollector {
    async fn on_event(&self, payload: &HookPayload) -> HookAction {
        match payload.hook_point {
            HookPoint::PostToolCall => {
                if let Some(tool_name) = &payload.tool_name {
                    let inv = ToolInvocation {
                        tool: tool_name.clone(),
                        arguments_summary: String::new(),
                        result_summary: payload.tool_result.as_deref().unwrap_or("").chars().take(200).collect(),
                    };
                    self.tool_buffer.lock().unwrap().push(inv);
                    *self.iteration_buffer.lock().unwrap() += 1;
                }
            }
            _ => {}
        }
        HookAction::allow()
    }

    fn name(&self) -> &str {
        "trace_collector"
    }
}

#[tokio::test]
async fn trace_collector_records_tool_chain() {
    let dir = TempDir::new().unwrap();
    let store = TraceStore::open(&dir.path().join("traces.sqlite3")).unwrap();
    let collector = TraceCollector::new(store);

    collector.on_skill_read("deploy", "deploy to staging");

    let payload = HookPayload::post_tool_call(None, 1, "terminal", "cargo build success");
    collector.on_event(&payload).await;

    let id = collector.finalize(TraceOutcome::Success, "deployed").unwrap();
    assert!(id > 0);

    let traces = collector.store.get_traces_for_skill("deploy", 10).unwrap();
    assert_eq!(traces.len(), 1);
    assert_eq!(traces[0].tool_chain.len(), 1);
    assert_eq!(traces[0].tool_chain[0].tool, "terminal");
}

#[tokio::test]
async fn trace_collector_ignores_without_active_skill() {
    let dir = TempDir::new().unwrap();
    let store = TraceStore::open(&dir.path().join("traces.sqlite3")).unwrap();
    let collector = TraceCollector::new(store);

    let payload = HookPayload::post_tool_call(None, 1, "terminal", "output");
    collector.on_event(&payload).await;

    let id = collector.finalize(TraceOutcome::Success, "done").unwrap();
    assert_eq!(id, 0);
}

#[tokio::test]
async fn trace_collector_accumulates_multiple_tools() {
    let dir = TempDir::new().unwrap();
    let store = TraceStore::open(&dir.path().join("traces.sqlite3")).unwrap();
    let collector = TraceCollector::new(store);

    collector.on_skill_read("multi", "complex task");

    collector.on_event(&HookPayload::post_tool_call(None, 1, "read_file", "contents")).await;
    collector.on_event(&HookPayload::post_tool_call(None, 2, "terminal", "built")).await;
    collector.on_event(&HookPayload::post_tool_call(None, 3, "write_file", "wrote")).await;

    collector.finalize(TraceOutcome::Success, "done").unwrap();

    let traces = collector.store.get_traces_for_skill("multi", 10).unwrap();
    assert_eq!(traces[0].tool_chain.len(), 3);
}
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test -p matrixclaw-tools skill_trace`
Expected: All previous + 3 new tests PASS

- [ ] **Step 3: Commit**

```bash
git add crates/matrixclaw-tools/src/builtin/skill_trace.rs
git commit -m "feat(tools): add TraceCollector lifecycle hook

TraceCollector observes PostToolCall events, buffers tool invocations
for the active skill, and records complete traces on finalize. Integrated
with LifecycleHook trait for agent loop injection."
```

---

## Task 3: TraceAnalyzer — Pattern Detection

**Files:**
- Create: `crates/matrixclaw-tools/src/builtin/skill_evolver.rs`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write the failing tests and implementation**

```rust
// In crates/matrixclaw-tools/src/builtin/skill_evolver.rs

use crate::builtin::skill_trace::{TraceOutcome, TraceStore};

#[derive(Debug, Clone)]
pub struct SkillHealth {
    pub skill_name: String,
    pub success_rate: f64,
    pub total_traces: usize,
    pub recent_failures: usize,
    pub needs_rewrite: bool,
}

#[derive(Debug, Clone)]
pub struct FailurePattern {
    pub tool_sequence: Vec<String>,
    pub occurrence_count: usize,
    pub example_task: String,
}

pub struct TraceAnalyzer {
    store: TraceStore,
    rewrite_threshold: f64,
    min_traces: usize,
    failure_window_hours: i64,
}

impl TraceAnalyzer {
    pub fn new(store: TraceStore) -> Self {
        Self {
            store,
            rewrite_threshold: 0.5,
            min_traces: 3,
            failure_window_hours: 24,
        }
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.rewrite_threshold = threshold;
        self
    }

    pub fn with_min_traces(mut self, min: usize) -> Self {
        self.min_traces = min;
        self
    }

    pub fn analyze_skill(&self, skill_name: &str) -> Result<SkillHealth, String> {
        let traces = self.store.get_traces_for_skill(skill_name, 50)?;
        let total = traces.len();
        if total < self.min_traces {
            return Ok(SkillHealth {
                skill_name: skill_name.to_string(),
                success_rate: 1.0,
                total_traces: total,
                recent_failures: 0,
                needs_rewrite: false,
            });
        }
        let rate = self.store.get_success_rate(skill_name, 50)?;
        let recent_failures = self.store.count_recent_failures(skill_name, self.failure_window_hours)?;
        let needs_rewrite = rate < self.rewrite_threshold && total >= self.min_traces;
        Ok(SkillHealth {
            skill_name: skill_name.to_string(),
            success_rate: rate,
            total_traces: total,
            recent_failures,
            needs_rewrite,
        })
    }

    pub fn find_failure_patterns(&self, skill_name: &str) -> Result<Vec<FailurePattern>, String> {
        let traces = self.store.get_traces_for_skill(skill_name, 50)?;
        let failures: Vec<_> = traces
            .iter()
            .filter(|t| t.outcome == TraceOutcome::Failure)
            .collect();
        if failures.is_empty() {
            return Ok(Vec::new());
        }

        let mut seq_counts: std::collections::HashMap<String, (usize, String)> =
            std::collections::HashMap::new();

        for fail in &failures {
            if fail.tool_chain.len() >= 2 {
                let seq: Vec<String> = fail.tool_chain.iter().map(|t| t.tool.clone()).collect();
                for window_len in 2..=seq.len().min(3) {
                    for i in 0..=seq.len() - window_len {
                        let key = seq[i..i + window_len].join(" -> ");
                        seq_counts
                            .entry(key)
                            .and_modify(|(c, _)| *c += 1)
                            .or_insert((1, fail.task_summary.clone()));
                    }
                }
            }
        }

        let mut patterns: Vec<FailurePattern> = seq_counts
            .into_iter()
            .filter(|(_, (count, _))| *count >= 2)
            .map(|(seq, (count, example))| FailurePattern {
                tool_sequence: seq.split(" -> ").map(String::from).collect(),
                occurrence_count: count,
                example_task: example,
            })
            .collect();
        patterns.sort_by(|a, b| b.occurrence_count.cmp(&a.occurrence_count));
        patterns.truncate(5);
        Ok(patterns)
    }

    pub fn gather_examples(
        &self,
        skill_name: &str,
        max_successes: usize,
        max_failures: usize,
    ) -> Result<(Vec<String>, Vec<String>), String> {
        let traces = self.store.get_traces_for_skill(skill_name, 50)?;
        let mut successes = Vec::new();
        let mut failures = Vec::new();

        for trace in &traces {
            let summary = format!(
                "Task: {}\nTools: {}\nOutcome: {:?}\nLLM snippet: {}",
                trace.task_summary,
                trace.tool_chain.iter().map(|t| &t.tool).collect::<Vec<_>>().join(", "),
                trace.outcome,
                &trace.llm_output_snippet.chars().take(200).collect::<String>(),
            );
            match trace.outcome {
                TraceOutcome::Success | TraceOutcome::Partial => {
                    if successes.len() < max_successes {
                        successes.push(summary);
                    }
                }
                TraceOutcome::Failure => {
                    if failures.len() < max_failures {
                        failures.push(summary);
                    }
                }
            }
        }
        Ok((successes, failures))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtin::skill_trace::{SkillTrace, ToolInvocation};
    use tempfile::TempDir;

    fn setup() -> (TraceAnalyzer, TempDir) {
        let dir = TempDir::new().unwrap();
        let store = TraceStore::open(&dir.path().join("traces.sqlite3")).unwrap();
        let analyzer = TraceAnalyzer::new(store);
        (analyzer, dir)
    }

    fn make_trace(skill: &str, task: &str, outcome: TraceOutcome, tools: &[&str]) -> SkillTrace {
        SkillTrace {
            id: 0,
            skill_name: skill.to_string(),
            task_summary: task.to_string(),
            outcome,
            tool_chain: tools
                .iter()
                .map(|t| ToolInvocation {
                    tool: t.to_string(),
                    arguments_summary: String::new(),
                    result_summary: String::new(),
                })
                .collect(),
            llm_output_snippet: "output".to_string(),
            iteration: 1,
            created_at: "2026-04-07 12:00:00".to_string(),
        }
    }

    #[test]
    fn healthy_skill_no_rewrite() {
        let (analyzer, _dir) = setup();
        for _ in 0..5 {
            analyzer.store.insert(&make_trace("good", "task", TraceOutcome::Success, &["terminal"])).unwrap();
        }
        let health = analyzer.analyze_skill("good").unwrap();
        assert!(!health.needs_rewrite);
        assert_eq!(health.success_rate, 1.0);
    }

    #[test]
    fn failing_skill_needs_rewrite() {
        let (analyzer, _dir) = setup();
        analyzer.store.insert(&make_trace("bad", "t1", TraceOutcome::Success, &["terminal"])).unwrap();
        analyzer.store.insert(&make_trace("bad", "t2", TraceOutcome::Failure, &["read_file", "terminal"])).unwrap();
        analyzer.store.insert(&make_trace("bad", "t3", TraceOutcome::Failure, &["read_file", "terminal"])).unwrap();
        analyzer.store.insert(&make_trace("bad", "t4", TraceOutcome::Failure, &["read_file", "terminal"])).unwrap();
        let health = analyzer.analyze_skill("bad").unwrap();
        assert!(health.needs_rewrite);
        assert!(health.success_rate < 0.5);
    }

    #[test]
    fn too_few_traces_no_rewrite() {
        let (analyzer, _dir) = setup();
        analyzer.store.insert(&make_trace("new", "t1", TraceOutcome::Failure, &["terminal"])).unwrap();
        let health = analyzer.analyze_skill("new").unwrap();
        assert!(!health.needs_rewrite);
    }

    #[test]
    fn find_failure_patterns_detects_repeated_sequence() {
        let (analyzer, _dir) = setup();
        for i in 0..3 {
            analyzer.store.insert(&make_trace("s", &format!("task {i}"), TraceOutcome::Failure, &["read_file", "terminal", "write_file"])).unwrap();
        }
        let patterns = analyzer.find_failure_patterns("s").unwrap();
        assert!(!patterns.is_empty());
        assert!(patterns[0].occurrence_count >= 2);
    }

    #[test]
    fn gather_examples_separates_success_failure() {
        let (analyzer, _dir) = setup();
        analyzer.store.insert(&make_trace("s", "good task", TraceOutcome::Success, &["terminal"])).unwrap();
        analyzer.store.insert(&make_trace("s", "bad task", TraceOutcome::Failure, &["read_file"])).unwrap();
        let (successes, failures) = analyzer.gather_examples("s", 5, 5).unwrap();
        assert_eq!(successes.len(), 1);
        assert_eq!(failures.len(), 1);
        assert!(successes[0].contains("good task"));
        assert!(failures[0].contains("bad task"));
    }

    #[test]
    fn no_failures_yields_empty_patterns() {
        let (analyzer, _dir) = setup();
        for _ in 0..3 {
            analyzer.store.insert(&make_trace("s", "t", TraceOutcome::Success, &["terminal"])).unwrap();
        }
        let patterns = analyzer.find_failure_patterns("s").unwrap();
        assert!(patterns.is_empty());
    }
}
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test -p matrixclaw-tools skill_evolver`
Expected: 6 tests PASS

- [ ] **Step 3: Register the module**

In `crates/matrixclaw-tools/src/builtin/mod.rs`, add:

```rust
pub mod skill_evolver;
```

- [ ] **Step 4: Commit**

```bash
git add crates/matrixclaw-tools/src/builtin/skill_evolver.rs crates/matrixclaw-tools/src/builtin/mod.rs
git commit -m "feat(tools): add TraceAnalyzer for skill health and failure patterns

TraceAnalyzer computes per-skill success rates, detects repeated failure
tool-chain patterns, and gathers success/failure examples for skill
rewriting. Triggers rewrite when success rate drops below threshold."
```

---

## Task 4: Skill Versioning

**Files:**
- Modify: `crates/matrixclaw-tools/src/builtin/skills.rs` (add `history` action, version-aware read/write)

- [ ] **Step 1: Write the failing tests and implementation**

Add version support to `SkillsTool`. When a skill is created/updated, the previous version is archived to `~/.matrixclaw/skills/<name>/v<N>.md`. The current version is always `SKILL.md`. A new `history` action lists versions.

Modify `SkillsTool` in `skills.rs` — add `history` action to the enum_values, and add these methods:

```rust
impl SkillsTool {
    fn handle_history(&self, call: &ToolCall) -> ToolResult {
        let name = match call.arguments.get("name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => return ToolResult::error(call, "missing required parameter: name"),
        };
        let skill_dir = self.skills_dir.join(name);
        if !skill_dir.exists() {
            return ToolResult::error(call, format!("skill not found: {name}"));
        }
        let mut versions: Vec<String> = Vec::new();
        if self.skill_entry(name).exists() {
            versions.push("current (SKILL.md)".to_string());
        }
        if let Ok(entries) = std::fs::read_dir(&skill_dir) {
            let mut archived: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().to_str().map(String::from))
                .filter(|n| n.starts_with('v') && n.ends_with(".md"))
                .collect();
            archived.sort();
            for v in archived {
                versions.push(v);
            }
        }
        if versions.is_empty() {
            ToolResult::success(call, "(no versions found)")
        } else {
            ToolResult::success(call, versions.join("\n"))
        }
    }

    fn archive_current(&self, name: &str) -> Result<(), String> {
        let skill_dir = self.skills_dir.join(name);
        let current = skill_dir.join("SKILL.md");
        if !current.exists() {
            return Ok(());
        }
        let existing_versions: Vec<String> = std::fs::read_dir(&skill_dir)
            .unwrap_or_else(|_| panic!("read_dir failed"))
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().to_str().map(String::from))
            .filter(|n| n.starts_with('v') && n.ends_with(".md"))
            .collect();
        let next_version = existing_versions.len() + 1;
        let archive_name = format!("v{next_version}.md");
        std::fs::copy(&current, skill_dir.join(&archive_name))
            .map_err(|e| format!("archive failed: {e}"))?;
        Ok(())
    }
}
```

Modify `handle_create` to archive before writing:

```rust
async fn handle_create(&self, call: &ToolCall) -> ToolResult {
    // ... existing validation ...

    let skill_dir = self.skills_dir.join(name);
    if let Err(e) = std::fs::create_dir_all(&skill_dir) {
        return ToolResult::error(call, format!("failed to create skill directory: {e}"));
    }
    if let Err(e) = self.archive_current(name) {
        return ToolResult::error(call, format!("failed to archive previous version: {e}"));
    }
    let entry_path = skill_dir.join("SKILL.md");
    if let Err(e) = std::fs::write(&entry_path, content) {
        return ToolResult::error(call, format!("failed to write skill: {e}"));
    }
    ToolResult::success(call, format!("created skill: {name}"))
}
```

Add `"history"` to the action enum_values, and add the match arm:

```rust
match action {
    "list" => self.handle_list(&call).await,
    "read" => self.handle_read(&call).await,
    "create" => self.handle_create(&call).await,
    "history" => self.handle_history(&call),
    _ => ToolResult::error(&call, format!("unknown action: {action}")),
}
```

Add tests:

```rust
#[tokio::test]
async fn create_archives_previous_version() {
    let (tool, _dir) = make_tool();
    call(&tool, r#"{"action":"create","name":"s","content":"v1"}"#).await;
    call(&tool, r#"{"action":"create","name":"s","content":"v2"}"#).await;
    let history = call(&tool, r#"{"action":"history","name":"s"}"#).await;
    assert!(!history.is_error);
    assert!(history.output.contains("current"));
    assert!(history.output.contains("v1.md"));
    let read = call(&tool, r#"{"action":"read","name":"s"}"#).await;
    assert_eq!(read.output, "v2");
}

#[tokio::test]
async fn history_shows_no_versions_for_new_skill() {
    let (tool, _dir) = make_tool();
    call(&tool, r#"{"action":"create","name":"s","content":"content"}"#).await;
    let history = call(&tool, r#"{"action":"history","name":"s"}"#).await;
    assert!(!history.is_error);
    assert!(history.output.contains("current"));
    assert!(!history.output.contains("v1.md"));
}

#[tokio::test]
async fn history_multiple_versions() {
    let (tool, _dir) = make_tool();
    call(&tool, r#"{"action":"create","name":"s","content":"v1"}"#).await;
    call(&tool, r#"{"action":"create","name":"s","content":"v2"}"#).await;
    call(&tool, r#"{"action":"create","name":"s","content":"v3"}"#).await;
    let history = call(&tool, r#"{"action":"history","name":"s"}"#).await;
    assert!(history.output.contains("v1.md"));
    assert!(history.output.contains("v2.md"));
    assert!(history.output.contains("current"));
}
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test -p matrixclaw-tools skills`
Expected: All existing + 3 new tests PASS

- [ ] **Step 3: Commit**

```bash
git add crates/matrixclaw-tools/src/builtin/skills.rs
git commit -m "feat(tools): skill versioning with archive-on-update

When a skill is created/updated, the previous version is archived to
v<N>.md. New 'history' action lists all versions. Enables rollback
and audit trail for self-evolving skills."
```

---

## Task 5: SkillRewriter — LLM-Driven Skill Improvement

**Files:**
- Modify: `crates/matrixclaw-tools/src/builtin/skill_evolver.rs` (add `SkillRewriter`)
- Modify: `crates/matrixclaw-tools/src/builtin/mod.rs` (add `skill_evolve` tool)
- Test: inline tests

- [ ] **Step 1: Write the SkillRewriter and skill_evolve tool**

The `SkillRewriter` uses the callback pattern (same as `SubagentRunner`) to avoid circular deps. Add to `skill_evolver.rs`:

```rust
use std::future::Future;
use std::pin::Pin;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};
use crate::builtin::skill_trace::TraceStore;

pub type LlmRewriteFn = Box<
    dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send>> + Send + Sync,
>;

pub struct SkillRewriter {
    analyzer: TraceAnalyzer,
    skills_dir: PathBuf,
    llm_call: Arc<LlmRewriteFn>,
}

impl SkillRewriter {
    pub fn new(
        trace_store: TraceStore,
        skills_dir: PathBuf,
        llm_call: LlmRewriteFn,
    ) -> Self {
        Self {
            analyzer: TraceAnalyzer::new(trace_store),
            skills_dir,
            llm_call: Arc::from(llm_call),
        }
    }

    pub async fn rewrite_skill(&self, skill_name: &str) -> Result<RewriteResult, String> {
        let health = self.analyzer.analyze_skill(skill_name)?;
        if !health.needs_rewrite {
            return Ok(RewriteResult {
                skill_name: skill_name.to_string(),
                rewritten: false,
                reason: format!("skill healthy (rate={:.0}%)", health.success_rate * 100.0),
            });
        }

        let (successes, failures) = self.analyzer.gather_examples(skill_name, 3, 5)?;
        let patterns = self.analyzer.find_failure_patterns(skill_name)?;

        let current_skill_path = self.skills_dir.join(skill_name).join("SKILL.md");
        let current_content = std::fs::read_to_string(&current_skill_path)
            .map_err(|e| format!("failed to read skill: {e}"))?;

        let patterns_text = if patterns.is_empty() {
            "No repeated tool-chain patterns detected.".to_string()
        } else {
            patterns
                .iter()
                .map(|p| {
                    format!(
                        "- Tools: {} (seen {} times, e.g. task: {})",
                        p.tool_sequence.join(" -> "),
                        p.occurrence_count,
                        p.example_task
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        let failures_text = if failures.is_empty() {
            "No failure examples available.".to_string()
        } else {
            failures
                .iter()
                .enumerate()
                .map(|(i, f)| format!("Failure {}:\n{}", i + 1, f))
                .collect::<Vec<_>>()
                .join("\n\n")
        };

        let successes_text = if successes.is_empty() {
            "No success examples available.".to_string()
        } else {
            successes
                .iter()
                .enumerate()
                .map(|(i, s)| format!("Success {}:\n{}", i + 1, s))
                .collect::<Vec<_>>()
                .join("\n\n")
        };

        let prompt = format!(
            r#"You are a skill improvement assistant. Your job is to rewrite a skill's instructions to improve its success rate.

## Current Skill Instructions
```
{current_content}
```

## Failure Analysis
Success rate: {:.0}% over {} traces
Recent failures (24h): {}

## Failure Patterns
{patterns_text}

## Failure Examples
{failures_text}

## Success Examples
{successes_text}

## Instructions
Rewrite the skill instructions above to:
1. Address the specific failure patterns identified
2. Preserve what works (reflected in success examples)
3. Add guardrails or alternative approaches for common failure points
4. Keep the skill concise and actionable

Output ONLY the improved skill content in Markdown. No explanations, no preamble."#,
            health.success_rate * 100.0,
            health.total_traces,
            health.recent_failures,
        );

        let rewritten = (self.llm_call)(prompt).await?;

        std::fs::write(&current_skill_path, &rewritten)
            .map_err(|e| format!("failed to write rewritten skill: {e}"))?;

        Ok(RewriteResult {
            skill_name: skill_name.to_string(),
            rewritten: true,
            reason: format!(
                "rewrote skill (was {:.0}% success, {} recent failures)",
                health.success_rate * 100.0,
                health.recent_failures
            ),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteResult {
    pub skill_name: String,
    pub rewritten: bool,
    pub reason: String,
}

pub struct SkillEvolveTool {
    descriptor: ToolDescriptor,
    rewriter: Arc<SkillRewriter>,
}

impl SkillEvolveTool {
    pub fn new(rewriter: Arc<SkillRewriter>) -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "skill_evolve",
                "Analyze a skill's execution history and rewrite it to improve success rate based on failure patterns.",
            )
            .with_parameters(vec![
                ToolParameter::required("skill_name", ParameterType::String, "Name of the skill to evolve"),
            ]),
            rewriter,
        }
    }
}

#[async_trait]
impl ToolExecutor for SkillEvolveTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let skill_name = match call.arguments.get("skill_name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => return ToolResult::error(&call, "missing required parameter: skill_name"),
        };
        match self.rewriter.rewrite_skill(skill_name).await {
            Ok(result) => {
                if result.rewritten {
                    ToolResult::success(&call, format!("skill evolved: {}", result.reason))
                } else {
                    ToolResult::success(&call, format!("no evolution needed: {}", result.reason))
                }
            }
            Err(e) => ToolResult::error(&call, format!("evolution failed: {e}")),
        }
    }
}
```

Add the `skill_evolve` tool to the `#[cfg(test)]` block in `skill_evolver.rs`:

```rust
#[tokio::test]
async fn rewriter_skips_healthy_skill() {
    let dir = TempDir::new().unwrap();
    let store = TraceStore::open(&dir.path().join("traces.sqlite3")).unwrap();
    for _ in 0..5 {
        store.insert(&make_trace("good", "t", TraceOutcome::Success, &["terminal"])).unwrap();
    }
    let skills_dir = dir.path().join("skills");
    std::fs::create_dir_all(skills_dir.join("good")).unwrap();
    std::fs::write(skills_dir.join("good").join("SKILL.md"), "be good").unwrap();

    let rewriter = SkillRewriter::new(store, skills_dir, Box::new(|_| {
        Box::pin(async { Err("should not be called".to_string()) })
    }));
    let result = rewriter.rewrite_skill("good").await.unwrap();
    assert!(!result.rewritten);
}

#[tokio::test]
async fn rewriter_calls_llm_for_failing_skill() {
    let dir = TempDir::new().unwrap();
    let store = TraceStore::open(&dir.path().join("traces.sqlite3")).unwrap();
    store.insert(&make_trace("bad", "t1", TraceOutcome::Success, &["terminal"])).unwrap();
    for i in 2..=4 {
        store.insert(&make_trace("bad", &format!("t{i}"), TraceOutcome::Failure, &["read_file", "terminal"])).unwrap();
    }
    let skills_dir = dir.path().join("skills");
    std::fs::create_dir_all(skills_dir.join("bad")).unwrap();
    std::fs::write(skills_dir.join("bad").join("SKILL.md"), "old instructions").unwrap();

    let rewriter = SkillRewriter::new(store, skills_dir, Box::new(|prompt| {
        Box::pin(async move {
            assert!(prompt.contains("old instructions"));
            assert!(prompt.contains("Failure"));
            Ok("improved instructions".to_string())
        })
    }));
    let result = rewriter.rewrite_skill("bad").await.unwrap();
    assert!(result.rewritten);
    let new_content = std::fs::read_to_string(skills_dir.join("bad").join("SKILL.md")).unwrap();
    assert_eq!(new_content, "improved instructions");
}
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test -p matrixclaw-tools skill_evolver`
Expected: All previous + 2 new tests PASS

- [ ] **Step 3: Commit**

```bash
git add crates/matrixclaw-tools/src/builtin/skill_evolver.rs
git commit -m "feat(tools): SkillRewriter with LLM-driven skill improvement

SkillRewriter analyzes failure patterns, gathers examples, constructs
a rewrite prompt, and calls the LLM to generate improved instructions.
Uses callback pattern (LlmRewriteFn) to avoid circular deps with
agent-core. Includes skill_evolve tool for manual triggering."
```

---

## Task 6: Wire TraceCollector into Agent Loop via Hooks

**Files:**
- Modify: `crates/app-host/src/live_runtime.rs` (wire TraceCollector into CompositeHook)
- Modify: `crates/app-host/src/chat.rs` (wire SkillRewriter callback, register skill_evolve tool)
- Test: integration test

- [ ] **Step 1: Wire TraceCollector in live_runtime.rs**

In `live_runtime.rs`, import and create the `TraceCollector`:

```rust
use matrixclaw_tools::builtin::skill_trace::{TraceCollector, TraceStore};
```

In the `LiveRunService` constructor or initialization, open the trace store and create the collector:

```rust
impl LiveRunService {
    pub fn new(home: &Path) -> Self {
        // ... existing initialization ...

        let trace_store = TraceStore::open(&TraceStore::db_path_for_home(home)).ok();
        if let Some(store) = trace_store {
            // TraceCollector will be added to hooks later when CompositeHook is built
        }

        // ... rest of init ...
    }
}
```

Add `TraceCollector` to the `CompositeHook` wherever it's constructed (in `chat.rs` where hooks are assembled):

```rust
let trace_store = TraceStore::open(&TraceStore::db_path_for_home(&home)).ok();
if let Some(store) = trace_store.clone() {
    let collector = TraceCollector::new(store);
    hooks.add(Box::new(collector));
}
```

- [ ] **Step 2: Wire SkillRewriter callback in chat.rs**

Create the `LlmRewriteFn` callback that calls the provider:

```rust
use matrixclaw_tools::builtin::skill_evolver::{LlmRewriteFn, SkillRewriter, SkillEvolveTool};
use matrixclaw_tools::builtin::skill_trace::TraceStore;
use std::sync::Arc;

fn make_llm_rewrite_fn(provider: Arc<Mutex<FallbackProvider>>) -> LlmRewriteFn {
    Box::new(move |prompt: String| {
        let provider = provider.clone();
        Box::pin(async move {
            let mut prov = provider.lock().map_err(|e| e.to_string())?;
            let request = RunRequest::new(prompt);
            let response = prov.complete(&request).await.map_err(|e| e.0)?;
            Ok(response.content.unwrap_or_default())
        })
    })
}
```

Register the `SkillEvolveTool`:

```rust
if let Some(store) = trace_store {
    let skills_dir = home.join(".matrixclaw").join("skills");
    let rewriter = SkillRewriter::new(store, skills_dir, make_llm_rewrite_fn(provider_clone));
    registry.register(Arc::new(SkillEvolveTool::new(Arc::new(rewriter))));
}
```

- [ ] **Step 3: Write integration test**

Add a test in `skill_evolver.rs` that tests the full flow:

```rust
#[tokio::test]
async fn full_evolution_flow() {
    let dir = TempDir::new().unwrap();
    let store = TraceStore::open(&dir.path().join("traces.sqlite3")).unwrap();

    // Seed: 1 success, 3 failures
    store.insert(&make_trace("deploy", "deploy staging", TraceOutcome::Success, &["terminal"])).unwrap();
    store.insert(&make_trace("deploy", "deploy prod fail 1", TraceOutcome::Failure, &["read_file", "terminal"])).unwrap();
    store.insert(&make_trace("deploy", "deploy prod fail 2", TraceOutcome::Failure, &["read_file", "terminal"])).unwrap();
    store.insert(&make_trace("deploy", "deploy prod fail 3", TraceOutcome::Failure, &["read_file", "terminal"])).unwrap();

    let skills_dir = dir.path().join("skills");
    std::fs::create_dir_all(skills_dir.join("deploy")).unwrap();
    std::fs::write(skills_dir.join("deploy").join("SKILL.md"), "# Deploy\nRun deploy command").unwrap();

    let rewriter = SkillRewriter::new(store.clone(), skills_dir.clone(), Box::new(|prompt| {
        Box::pin(async move {
            assert!(prompt.contains("Deploy"));
            assert!(prompt.contains("read_file -> terminal"));
            Ok("# Deploy (v2)\nImproved deploy with error handling".to_string())
        })
    }));

    let tool = SkillEvolveTool::new(Arc::new(rewriter));
    let call = ToolCall::new("1".into(), "skill_evolve".into(), serde_json::json!({"skill_name": "deploy"}));
    let result = tool.execute(call).await;
    assert!(!result.is_error);
    assert!(result.output.contains("evolved"));

    let new_content = std::fs::read_to_string(skills_dir.join("deploy").join("SKILL.md")).unwrap();
    assert!(new_content.contains("Improved deploy"));
}
```

- [ ] **Step 4: Run full workspace test**

Run: `cargo check --workspace && cargo test --workspace && cargo clippy --workspace --all-targets && cargo fmt --all -- --check`
Expected: All tests PASS, clippy clean, fmt clean

- [ ] **Step 5: Commit**

```bash
git add crates/app-host/src/live_runtime.rs crates/app-host/src/chat.rs crates/matrixclaw-tools/src/builtin/skill_evolver.rs
git commit -m "feat(app): wire TraceCollector and SkillRewriter into runtime

TraceCollector hooked into agent loop via CompositeHook. SkillRewriter
uses provider callback for LLM-based skill rewriting. skill_evolve tool
registered in chat mode. Full evolution flow tested end-to-end."
```

---

## Task 7: Update register_all and Module Exports

**Files:**
- Modify: `crates/matrixclaw-tools/src/builtin/mod.rs` (export new modules, update register_all if needed)
- Modify: `crates/matrixclaw-tools/src/lib.rs` (export new types)
- Run full validation

- [ ] **Step 1: Ensure module declarations in mod.rs**

In `crates/matrixclaw-tools/src/builtin/mod.rs`, verify these modules are declared:

```rust
pub mod skill_trace;
pub mod skill_evolver;
```

- [ ] **Step 2: Export key types from lib.rs if needed**

In `crates/matrixclaw-tools/src/lib.rs`, add any re-exports needed by `app-host`:

```rust
pub use builtin::skill_trace::{TraceCollector, TraceStore, SkillTrace, TraceOutcome, ToolInvocation};
pub use builtin::skill_evolver::{SkillRewriter, SkillEvolveTool, TraceAnalyzer, LlmRewriteFn};
```

Only if `app-host` needs them directly. If `app-host` uses the full path `matrixclaw_tools::builtin::skill_trace::...`, this step can be skipped.

- [ ] **Step 3: Run full validation**

Run: `cargo check --workspace && cargo test --workspace && cargo clippy --workspace --all-targets && cargo fmt --all -- --check`
Expected: ALL CLEAN

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "chore: export skill trace and evolver types

Add module declarations and re-exports for skill_trace and skill_evolver.
Clean up any unused imports flagged by clippy."
```

---

## Task 8: Update Documentation

**Files:**
- Modify: `docs/plans/runtime-rethink.md` (mark self-evolving skills complete)
- Modify: `DESIGN.md` (add skill evolution section)

- [ ] **Step 1: Update runtime-rethink.md**

In Phase 8 section, change:
```markdown
- [ ] Self-evolving skills: DSPy-style skill improvement from execution feedback
```
to:
```markdown
- [x] Self-evolving skills: Rust-native GePA+MiProv2 with TraceCollector, TraceAnalyzer, SkillRewriter
```

- [ ] **Step 2: Add skill evolution section to DESIGN.md**

Add a section describing the architecture:

```markdown
### Self-Evolving Skills

Skills automatically improve from execution feedback through three components:

1. **TraceCollector** (LifecycleHook) — observes every tool call after a `skills.read`, records the complete tool chain and outcome (success/failure/partial) into `skill_traces.sqlite3`
2. **TraceAnalyzer** — groups traces by skill, computes success rates, detects repeated failure patterns in tool-chain sequences, gathers success/failure examples
3. **SkillRewriter** — when a skill's success rate drops below 50% with 3+ traces, constructs a rewrite prompt containing the current skill + failure patterns + examples, calls the LLM to generate improved instructions, writes as new version with archive

**Key design decisions:**
- No Python dependency — pure Rust implementation inspired by DSPy's GePA and MiProv2
- Callback pattern for LLM calls avoids circular dependency between matrixclaw-tools and agent-core
- Skill versioning archives previous versions to v<N>.md before rewriting
- Triggered automatically via lifecycle hooks or manually via `skill_evolve` tool
```

- [ ] **Step 3: Commit**

```bash
git add docs/plans/runtime-rethink.md DESIGN.md
git commit -m "docs: add self-evolving skills to roadmap and design docs

Update Phase 8 status, add skill evolution architecture section
to DESIGN.md covering TraceCollector, TraceAnalyzer, SkillRewriter."
```

---

## Summary

| Task | Component | LOC Est. | Tests |
|------|-----------|----------|-------|
| 1 | SkillTrace + TraceStore | ~180 | 7 |
| 2 | TraceCollector hook | ~80 | 3 |
| 3 | TraceAnalyzer | ~160 | 6 |
| 4 | Skill versioning | ~60 | 3 |
| 5 | SkillRewriter + skill_evolve | ~200 | 2 |
| 6 | Wire into runtime | ~60 | 1 |
| 7 | Exports + cleanup | ~20 | 0 |
| 8 | Documentation | ~30 | 0 |
| **Total** | | **~790** | **22** |
