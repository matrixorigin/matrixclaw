use std::path::{Path, PathBuf};
use std::sync::Mutex;

use async_trait::async_trait;
use matrixclaw_hooks::{HookAction, HookPayload, HookPoint, LifecycleHook};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

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

pub struct TraceStore {
    db: Mutex<Connection>,
}

impl TraceStore {
    pub fn open(db_path: &Path) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("skill_trace: failed to create db directory: {e}"))?;
        }
        let conn = Connection::open(db_path)
            .map_err(|e| format!("skill_trace: failed to open db: {e}"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS skill_traces (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                skill_name TEXT NOT NULL,
                task_summary TEXT NOT NULL,
                outcome TEXT NOT NULL,
                tool_chain TEXT NOT NULL,
                llm_output_snippet TEXT NOT NULL DEFAULT '',
                iteration INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_skill_traces_skill_name ON skill_traces(skill_name);
            CREATE INDEX IF NOT EXISTS idx_skill_traces_outcome ON skill_traces(outcome);
            CREATE VIRTUAL TABLE IF NOT EXISTS traces_fts USING fts5(task_summary, content=skill_traces, content_rowid=id);
            CREATE TRIGGER IF NOT EXISTS traces_fts_ai AFTER INSERT ON skill_traces BEGIN
                INSERT INTO traces_fts(rowid, task_summary) VALUES (new.id, new.task_summary);
            END;
            CREATE TRIGGER IF NOT EXISTS traces_fts_ad AFTER DELETE ON skill_traces BEGIN
                INSERT INTO traces_fts(traces_fts, rowid, task_summary) VALUES('delete', old.id, old.task_summary);
            END;
            CREATE TRIGGER IF NOT EXISTS traces_fts_au AFTER UPDATE ON skill_traces BEGIN
                INSERT INTO traces_fts(traces_fts, rowid, task_summary) VALUES('delete', old.id, old.task_summary);
                INSERT INTO traces_fts(rowid, task_summary) VALUES (new.id, new.task_summary);
            END;",
        )
        .map_err(|e| format!("skill_trace: failed to create tables: {e}"))?;
        Ok(Self {
            db: Mutex::new(conn),
        })
    }

    pub fn db_path_for_home(home: &Path) -> PathBuf {
        home.join(".matrixclaw")
            .join("state")
            .join("skill_traces.sqlite3")
    }

    pub fn insert(&self, trace: &SkillTrace) -> Result<i64, String> {
        let db = self.db.lock().map_err(|e| format!("db lock: {e}"))?;
        let tool_chain_json = serde_json::to_string(&trace.tool_chain)
            .map_err(|e| format!("serialize tool_chain: {e}"))?;
        let outcome_str = serde_json::to_value(&trace.outcome)
            .map_err(|e| format!("serialize outcome: {e}"))?
            .as_str()
            .unwrap_or("success")
            .to_string();
        db.execute(
            "INSERT INTO skill_traces (skill_name, task_summary, outcome, tool_chain, llm_output_snippet, iteration, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                trace.skill_name,
                trace.task_summary,
                outcome_str,
                tool_chain_json,
                trace.llm_output_snippet,
                trace.iteration,
                trace.created_at,
            ],
        )
        .map_err(|e| format!("insert failed: {e}"))?;
        Ok(db.last_insert_rowid())
    }

    pub fn get_traces_for_skill(
        &self,
        skill_name: &str,
        limit: usize,
    ) -> Result<Vec<SkillTrace>, String> {
        let db = self.db.lock().map_err(|e| format!("db lock: {e}"))?;
        let mut stmt = db
            .prepare("SELECT id, skill_name, task_summary, outcome, tool_chain, llm_output_snippet, iteration, created_at FROM skill_traces WHERE skill_name = ?1 ORDER BY created_at DESC LIMIT ?2")
            .map_err(|e| format!("prepare failed: {e}"))?;
        let rows = stmt
            .query_map(params![skill_name, limit], |row| {
                let id: i64 = row.get(0)?;
                let skill_name: String = row.get(1)?;
                let task_summary: String = row.get(2)?;
                let outcome_str: String = row.get(3)?;
                let tool_chain_json: String = row.get(4)?;
                let llm_output_snippet: String = row.get(5)?;
                let iteration: u32 = row.get(6)?;
                let created_at: String = row.get(7)?;
                Ok((
                    id,
                    skill_name,
                    task_summary,
                    outcome_str,
                    tool_chain_json,
                    llm_output_snippet,
                    iteration,
                    created_at,
                ))
            })
            .map_err(|e| format!("query failed: {e}"))?;
        let mut traces = Vec::new();
        for row in rows {
            let (
                id,
                skill_name,
                task_summary,
                outcome_str,
                tool_chain_json,
                llm_output_snippet,
                iteration,
                created_at,
            ) = row.map_err(|e| format!("row failed: {e}"))?;
            let outcome: TraceOutcome =
                serde_json::from_value(serde_json::Value::String(outcome_str))
                    .map_err(|e| format!("deserialize outcome: {e}"))?;
            let tool_chain: Vec<ToolInvocation> = serde_json::from_str(&tool_chain_json)
                .map_err(|e| format!("deserialize tool_chain: {e}"))?;
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
        let db = self.db.lock().map_err(|e| format!("db lock: {e}"))?;
        let total: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM skill_traces WHERE skill_name = ?1 ORDER BY created_at DESC LIMIT ?2",
                params![skill_name, last_n as i64],
                |row| row.get(0),
            )
            .map_err(|e| format!("count failed: {e}"))?;
        if total == 0 {
            return Ok(1.0);
        }
        let successes: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM skill_traces WHERE skill_name = ?1 AND outcome = 'success' ORDER BY created_at DESC LIMIT ?2",
                params![skill_name, last_n as i64],
                |row| row.get(0),
            )
            .map_err(|e| format!("count successes failed: {e}"))?;
        Ok(successes as f64 / total as f64)
    }

    pub fn count_recent_failures(
        &self,
        skill_name: &str,
        since_hours: i64,
    ) -> Result<usize, String> {
        let db = self.db.lock().map_err(|e| format!("db lock: {e}"))?;
        let count: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM skill_traces WHERE skill_name = ?1 AND outcome = 'failure' AND created_at >= datetime('now', ?2)",
                params![skill_name, format!("-{since_hours} hours")],
                |row| row.get(0),
            )
            .map_err(|e| format!("count recent failures failed: {e}"))?;
        Ok(count as usize)
    }

    pub fn search_traces(&self, query: &str, limit: usize) -> Result<Vec<SkillTrace>, String> {
        let db = self.db.lock().map_err(|e| format!("db lock: {e}"))?;
        let mut stmt = db
            .prepare(
                "SELECT s.id, s.skill_name, s.task_summary, s.outcome, s.tool_chain, s.llm_output_snippet, s.iteration, s.created_at FROM skill_traces s JOIN traces_fts f ON f.rowid = s.id WHERE traces_fts MATCH ?1 ORDER BY rank LIMIT ?2",
            )
            .map_err(|e| format!("prepare failed: {e}"))?;
        let rows = stmt
            .query_map(params![query, limit as i64], |row| {
                let id: i64 = row.get(0)?;
                let skill_name: String = row.get(1)?;
                let task_summary: String = row.get(2)?;
                let outcome_str: String = row.get(3)?;
                let tool_chain_json: String = row.get(4)?;
                let llm_output_snippet: String = row.get(5)?;
                let iteration: u32 = row.get(6)?;
                let created_at: String = row.get(7)?;
                Ok((
                    id,
                    skill_name,
                    task_summary,
                    outcome_str,
                    tool_chain_json,
                    llm_output_snippet,
                    iteration,
                    created_at,
                ))
            })
            .map_err(|e| format!("query failed: {e}"))?;
        let mut traces = Vec::new();
        for row in rows {
            let (
                id,
                skill_name,
                task_summary,
                outcome_str,
                tool_chain_json,
                llm_output_snippet,
                iteration,
                created_at,
            ) = row.map_err(|e| format!("row failed: {e}"))?;
            let outcome: TraceOutcome =
                serde_json::from_value(serde_json::Value::String(outcome_str))
                    .map_err(|e| format!("deserialize outcome: {e}"))?;
            let tool_chain: Vec<ToolInvocation> = serde_json::from_str(&tool_chain_json)
                .map_err(|e| format!("deserialize tool_chain: {e}"))?;
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
}

pub struct TraceCollector {
    store: TraceStore,
    active_skill: Mutex<Option<String>>,
    active_task: Mutex<Option<String>>,
    tool_buffer: Mutex<Vec<ToolInvocation>>,
    iteration_buffer: Mutex<u32>,
}

impl TraceCollector {
    pub fn new(store: TraceStore) -> Self {
        Self {
            store,
            active_skill: Mutex::new(None),
            active_task: Mutex::new(None),
            tool_buffer: Mutex::new(Vec::new()),
            iteration_buffer: Mutex::new(0),
        }
    }

    pub fn on_skill_read(&self, skill_name: &str, task_summary: &str) {
        *self.active_skill.lock().unwrap() = Some(skill_name.to_string());
        *self.active_task.lock().unwrap() = Some(task_summary.to_string());
        self.tool_buffer.lock().unwrap().clear();
        *self.iteration_buffer.lock().unwrap() = 0;
    }

    pub fn finalize(&self, outcome: TraceOutcome, llm_snippet: &str) -> Result<i64, String> {
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
                    created_at: now_iso(),
                };
                self.store.insert(&trace)
            }
            _ => Ok(0),
        }
    }
}

fn now_iso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;
    let year = 1970 + (days / 365);
    let day_of_year = days % 365;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;
    format!("{year}-{month:02}-{day:02} {hours:02}:{minutes:02}:{seconds:02}")
}

#[async_trait]
impl LifecycleHook for TraceCollector {
    async fn on_event(&self, payload: &HookPayload) -> HookAction {
        if payload.hook_point == HookPoint::PostToolCall {
            if let Some(tool_name) = &payload.tool_name {
                let inv = ToolInvocation {
                    tool: tool_name.clone(),
                    arguments_summary: String::new(),
                    result_summary: payload
                        .tool_result
                        .as_deref()
                        .unwrap_or("")
                        .chars()
                        .take(200)
                        .collect(),
                };
                self.tool_buffer.lock().unwrap().push(inv);
                *self.iteration_buffer.lock().unwrap() += 1;
            }
        }
        HookAction::allow()
    }

    fn name(&self) -> &str {
        "trace_collector"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_store() -> (TraceStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test-traces.sqlite3");
        let store = TraceStore::open(&db_path).unwrap();
        (store, dir)
    }

    fn make_trace(skill_name: &str, outcome: TraceOutcome, task_summary: &str) -> SkillTrace {
        SkillTrace {
            id: 0,
            skill_name: skill_name.to_string(),
            task_summary: task_summary.to_string(),
            outcome,
            tool_chain: vec![],
            llm_output_snippet: String::new(),
            iteration: 1,
            created_at: "2025-01-01T00:00:00".to_string(),
        }
    }

    #[test]
    fn insert_and_retrieve() {
        let (store, _dir) = make_store();
        let trace = make_trace("deploy", TraceOutcome::Success, "deploy to prod");
        let id = store.insert(&trace).unwrap();
        let traces = store.get_traces_for_skill("deploy", 10).unwrap();
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].id, id);
        assert_eq!(traces[0].skill_name, "deploy");
        assert_eq!(traces[0].outcome, TraceOutcome::Success);
    }

    #[test]
    fn success_rate_calculation() {
        let (store, _dir) = make_store();
        store
            .insert(&make_trace("build", TraceOutcome::Success, "build project"))
            .unwrap();
        store
            .insert(&make_trace(
                "build",
                TraceOutcome::Success,
                "build project again",
            ))
            .unwrap();
        store
            .insert(&make_trace("build", TraceOutcome::Failure, "build failed"))
            .unwrap();
        let rate = store.get_success_rate("build", 10).unwrap();
        assert!((rate - 0.6667).abs() < 0.01);
    }

    #[test]
    fn success_rate_empty_is_one() {
        let (store, _dir) = make_store();
        let rate = store.get_success_rate("nonexistent", 10).unwrap();
        assert_eq!(rate, 1.0);
    }

    #[test]
    fn count_recent_failures() {
        let (store, _dir) = make_store();
        let mut fail_trace = make_trace("test", TraceOutcome::Failure, "test failed");
        fail_trace.created_at = "2025-01-01T00:00:00".to_string();
        store.insert(&fail_trace).unwrap();
        let success_trace = make_trace("test", TraceOutcome::Success, "test passed");
        store.insert(&success_trace).unwrap();
        let count = store.count_recent_failures("test", 876000).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn search_traces_by_task() {
        let (store, _dir) = make_store();
        store
            .insert(&make_trace(
                "deploy",
                TraceOutcome::Success,
                "deploy k8s pods to cluster",
            ))
            .unwrap();
        store
            .insert(&make_trace(
                "deploy",
                TraceOutcome::Success,
                "deploy to aws",
            ))
            .unwrap();
        let results = store.search_traces("k8s", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].task_summary.contains("k8s"));
    }

    #[test]
    fn tool_chain_roundtrip() {
        let (store, _dir) = make_store();
        let mut trace = make_trace("debug", TraceOutcome::Partial, "debug memory leak");
        trace.tool_chain = vec![
            ToolInvocation {
                tool: "terminal".to_string(),
                arguments_summary: "grep memory log".to_string(),
                result_summary: "found leak in module x".to_string(),
            },
            ToolInvocation {
                tool: "filesystem".to_string(),
                arguments_summary: "read src/module.rs".to_string(),
                result_summary: "file contents".to_string(),
            },
        ];
        store.insert(&trace).unwrap();
        let traces = store.get_traces_for_skill("debug", 10).unwrap();
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].tool_chain.len(), 2);
        assert_eq!(traces[0].tool_chain[0].tool, "terminal");
        assert_eq!(
            traces[0].tool_chain[1].arguments_summary,
            "read src/module.rs"
        );
    }

    #[tokio::test]
    async fn trace_collector_records_tool_chain() {
        let dir = TempDir::new().unwrap();
        let store = TraceStore::open(&dir.path().join("tc.sqlite3")).unwrap();
        let collector = TraceCollector::new(store);
        collector.on_skill_read("deploy", "deploy to staging");
        let payload = HookPayload::post_tool_call(None, 1, "terminal", "cargo build success");
        collector.on_event(&payload).await;
        let id = collector
            .finalize(TraceOutcome::Success, "deployed")
            .unwrap();
        assert!(id > 0);
        let traces = collector.store.get_traces_for_skill("deploy", 10).unwrap();
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].tool_chain.len(), 1);
        assert_eq!(traces[0].tool_chain[0].tool, "terminal");
    }

    #[tokio::test]
    async fn trace_collector_ignores_without_active_skill() {
        let dir = TempDir::new().unwrap();
        let store = TraceStore::open(&dir.path().join("tc2.sqlite3")).unwrap();
        let collector = TraceCollector::new(store);
        let payload = HookPayload::post_tool_call(None, 1, "terminal", "output");
        collector.on_event(&payload).await;
        let id = collector.finalize(TraceOutcome::Success, "done").unwrap();
        assert_eq!(id, 0);
    }

    #[tokio::test]
    async fn trace_collector_accumulates_multiple_tools() {
        let dir = TempDir::new().unwrap();
        let store = TraceStore::open(&dir.path().join("tc3.sqlite3")).unwrap();
        let collector = TraceCollector::new(store);
        collector.on_skill_read("multi", "complex task");
        collector
            .on_event(&HookPayload::post_tool_call(
                None,
                1,
                "read_file",
                "contents",
            ))
            .await;
        collector
            .on_event(&HookPayload::post_tool_call(None, 2, "terminal", "built"))
            .await;
        collector
            .on_event(&HookPayload::post_tool_call(None, 3, "write_file", "wrote"))
            .await;
        collector.finalize(TraceOutcome::Success, "done").unwrap();
        let traces = collector.store.get_traces_for_skill("multi", 10).unwrap();
        assert_eq!(traces[0].tool_chain.len(), 3);
    }
}
