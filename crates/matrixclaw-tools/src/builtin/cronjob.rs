use std::path::{Path, PathBuf};
use std::sync::Mutex;

use async_trait::async_trait;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub id: Option<i64>,
    pub name: String,
    pub schedule: String,
    pub task_prompt: String,
    pub enabled: bool,
    pub last_run: Option<i64>,
    pub next_run: Option<i64>,
}

pub struct CronStore {
    db: Connection,
}

impl CronStore {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cron: failed to create db directory: {e}"))?;
        }
        let conn = Connection::open(path).map_err(|e| format!("cron: failed to open db: {e}"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS cron_jobs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                schedule TEXT NOT NULL,
                task_prompt TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                last_run INTEGER,
                next_run INTEGER
            );",
        )
        .map_err(|e| format!("cron: failed to create table: {e}"))?;

        Ok(Self { db: conn })
    }

    fn row_to_job(row: &rusqlite::Row<'_>) -> rusqlite::Result<CronJob> {
        Ok(CronJob {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            schedule: row.get(2)?,
            task_prompt: row.get(3)?,
            enabled: row.get::<_, i32>(4)? != 0,
            last_run: row.get(5)?,
            next_run: row.get(6)?,
        })
    }

    pub fn create_job(&self, job: &CronJob) -> Result<i64, String> {
        let next_run = parse_schedule_to_next(&job.schedule, now_epoch())?;
        self.db
            .execute(
                "INSERT INTO cron_jobs (name, schedule, task_prompt, enabled, last_run, next_run) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![job.name, job.schedule, job.task_prompt, job.enabled as i32, job.last_run, next_run],
            )
            .map_err(|e| format!("cron: create failed: {e}"))?;
        Ok(self.db.last_insert_rowid())
    }

    pub fn list_jobs(&self) -> Result<Vec<CronJob>, String> {
        let mut stmt = self
            .db
            .prepare("SELECT id, name, schedule, task_prompt, enabled, last_run, next_run FROM cron_jobs ORDER BY id")
            .map_err(|e| format!("cron: list failed: {e}"))?;
        let rows = stmt
            .query_map([], Self::row_to_job)
            .map_err(|e| format!("cron: list failed: {e}"))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn remove_job_by_name(&self, name: &str) -> Result<(), String> {
        let changed = self
            .db
            .execute("DELETE FROM cron_jobs WHERE name = ?1", params![name])
            .map_err(|e| format!("cron: remove failed: {e}"))?;
        if changed == 0 {
            return Err(format!("cron: job not found: {name}"));
        }
        Ok(())
    }

    pub fn update_job_by_name(
        &self,
        name: &str,
        schedule: Option<&str>,
        task: Option<&str>,
        enabled: Option<bool>,
    ) -> Result<(), String> {
        if let Some(s) = schedule {
            let next_run = parse_schedule_to_next(s, now_epoch())?;
            self.db
                .execute(
                    "UPDATE cron_jobs SET schedule = ?1, next_run = ?2 WHERE name = ?3",
                    params![s, next_run, name],
                )
                .map_err(|e| format!("cron: update failed: {e}"))?;
        }
        if let Some(t) = task {
            self.db
                .execute(
                    "UPDATE cron_jobs SET task_prompt = ?1 WHERE name = ?2",
                    params![t, name],
                )
                .map_err(|e| format!("cron: update failed: {e}"))?;
        }
        if let Some(e) = enabled {
            self.db
                .execute(
                    "UPDATE cron_jobs SET enabled = ?1 WHERE name = ?2",
                    params![e as i32, name],
                )
                .map_err(|e| format!("cron: update failed: {e}"))?;
        }
        Ok(())
    }

    pub fn get_due_jobs(&self, now: i64) -> Result<Vec<CronJob>, String> {
        let mut stmt = self
            .db
            .prepare(
                "SELECT id, name, schedule, task_prompt, enabled, last_run, next_run FROM cron_jobs WHERE enabled = 1 AND next_run <= ?1",
            )
            .map_err(|e| format!("cron: get_due failed: {e}"))?;
        let rows = stmt
            .query_map(params![now], Self::row_to_job)
            .map_err(|e| format!("cron: get_due failed: {e}"))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn mark_run(&self, id: i64, now: i64, next_run: i64) -> Result<(), String> {
        self.db
            .execute(
                "UPDATE cron_jobs SET last_run = ?1, next_run = ?2 WHERE id = ?3",
                params![now, next_run, id],
            )
            .map_err(|e| format!("cron: mark_run failed: {e}"))?;
        Ok(())
    }

    pub fn find_by_name(&self, name: &str) -> Result<Option<CronJob>, String> {
        let mut stmt = self
            .db
            .prepare(
                "SELECT id, name, schedule, task_prompt, enabled, last_run, next_run FROM cron_jobs WHERE name = ?1",
            )
            .map_err(|e| format!("cron: find failed: {e}"))?;
        let result = stmt.query_row(params![name], Self::row_to_job).ok();
        Ok(result)
    }
}

pub fn now_epoch() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub fn parse_interval(schedule: &str) -> Result<i64, String> {
    let s = schedule.trim().to_lowercase();
    let s = s.strip_prefix("every ").ok_or_else(|| {
        format!(
            "invalid schedule '{schedule}': must be in the form 'every <N><unit>' (e.g. 'every 30m')"
        )
    })?;
    let s = s.trim();
    let (num_part, unit_part) = s.chars().partition::<String, _>(|c| c.is_ascii_digit());
    let num: i64 = num_part
        .parse()
        .map_err(|_| format!("invalid schedule '{schedule}': expected number"))?;
    match unit_part.as_str() {
        "s" => Ok(num),
        "m" => Ok(num * 60),
        "h" => Ok(num * 3600),
        "d" => Ok(num * 86400),
        _ => Err(format!(
            "invalid schedule '{schedule}': unknown unit '{unit_part}'. Use s, m, h, or d"
        )),
    }
}

pub fn parse_schedule_to_next(schedule: &str, now: i64) -> Result<i64, String> {
    let secs = parse_interval(schedule)?;
    Ok(now + secs)
}

pub struct CronjobTool {
    descriptor: ToolDescriptor,
    store: Mutex<CronStore>,
}

impl CronjobTool {
    pub fn open(db_path: &Path) -> Result<Self, String> {
        let store = CronStore::open(db_path)?;
        Ok(Self {
            descriptor: ToolDescriptor::new(
                "cronjob",
                "Manage scheduled tasks. Create, list, remove, and update cron jobs that run on simple intervals.",
            )
            .with_parameters(vec![
                ToolParameter::required("action", ParameterType::String, "Action to perform")
                    .enum_values(&["create", "list", "remove", "update"]),
                ToolParameter::optional("name", ParameterType::String, "Job name"),
                ToolParameter::optional(
                    "schedule",
                    ParameterType::String,
                    "Interval schedule (e.g. 'every 30m', 'every 1h', 'every 24h')",
                ),
                ToolParameter::optional("task", ParameterType::String, "Task prompt to execute"),
                ToolParameter::optional(
                    "enabled",
                    ParameterType::Boolean,
                    "Whether the job is enabled",
                ),
            ]),
            store: Mutex::new(store),
        })
    }

    pub fn db_path_for_home(home: &Path) -> PathBuf {
        home.join(".matrixclaw").join("state").join("cron.db")
    }
}

trait EnumValues {
    fn enum_values(self, values: &[&str]) -> Self;
}

impl EnumValues for ToolParameter {
    fn enum_values(mut self, values: &[&str]) -> Self {
        self.enum_values = Some(values.iter().map(|s| s.to_string()).collect());
        self
    }
}

#[async_trait]
impl ToolExecutor for CronjobTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let action = match call.arguments.get("action").and_then(|v| v.as_str()) {
            Some(a) => a,
            None => return ToolResult::error(&call, "missing required parameter: action"),
        };

        let store = match self.store.lock() {
            Ok(g) => g,
            Err(e) => return ToolResult::error(&call, format!("cron db lock: {e}")),
        };

        match action {
            "create" => {
                let name = match call.arguments.get("name").and_then(|v| v.as_str()) {
                    Some(n) => n,
                    None => return ToolResult::error(&call, "missing required parameter: name"),
                };
                let schedule = match call.arguments.get("schedule").and_then(|v| v.as_str()) {
                    Some(s) => s,
                    None => {
                        return ToolResult::error(&call, "missing required parameter: schedule")
                    }
                };
                let task = match call.arguments.get("task").and_then(|v| v.as_str()) {
                    Some(t) => t,
                    None => return ToolResult::error(&call, "missing required parameter: task"),
                };
                if let Err(e) = parse_schedule_to_next(schedule, 0) {
                    return ToolResult::error(&call, e);
                }
                let job = CronJob {
                    id: None,
                    name: name.to_string(),
                    schedule: schedule.to_string(),
                    task_prompt: task.to_string(),
                    enabled: true,
                    last_run: None,
                    next_run: None,
                };
                match store.create_job(&job) {
                    Ok(_) => ToolResult::success(&call, format!("created job: {name}")),
                    Err(e) => ToolResult::error(&call, e),
                }
            }
            "list" => {
                let jobs = match store.list_jobs() {
                    Ok(j) => j,
                    Err(e) => return ToolResult::error(&call, e),
                };
                if jobs.is_empty() {
                    return ToolResult::success(&call, "(no cron jobs)");
                }
                let lines: Vec<String> = jobs
                    .iter()
                    .map(|j| {
                        let status = if j.enabled { "enabled" } else { "disabled" };
                        let next = j
                            .next_run
                            .map(|t| t.to_string())
                            .unwrap_or_else(|| "-".to_string());
                        format!(
                            "- {} [{}] schedule={} next_run={}",
                            j.name, status, j.schedule, next
                        )
                    })
                    .collect();
                ToolResult::success(&call, lines.join("\n"))
            }
            "remove" => {
                let name = match call.arguments.get("name").and_then(|v| v.as_str()) {
                    Some(n) => n,
                    None => return ToolResult::error(&call, "missing required parameter: name"),
                };
                match store.remove_job_by_name(name) {
                    Ok(()) => ToolResult::success(&call, format!("removed job: {name}")),
                    Err(e) => ToolResult::error(&call, e),
                }
            }
            "update" => {
                let name = match call.arguments.get("name").and_then(|v| v.as_str()) {
                    Some(n) => n,
                    None => return ToolResult::error(&call, "missing required parameter: name"),
                };
                let schedule = call.arguments.get("schedule").and_then(|v| v.as_str());
                let task = call.arguments.get("task").and_then(|v| v.as_str());
                let enabled = call.arguments.get("enabled").and_then(|v| v.as_bool());
                if let Some(s) = schedule {
                    if let Err(e) = parse_schedule_to_next(s, 0) {
                        return ToolResult::error(&call, e);
                    }
                }
                match store.update_job_by_name(name, schedule, task, enabled) {
                    Ok(()) => ToolResult::success(&call, format!("updated job: {name}")),
                    Err(e) => ToolResult::error(&call, e),
                }
            }
            _ => ToolResult::error(&call, format!("unknown action: {action}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_tool() -> (CronjobTool, TempDir) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("cron.db");
        let tool = CronjobTool::open(&db_path).unwrap();
        (tool, dir)
    }

    fn make_store() -> (CronStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("cron.db");
        let store = CronStore::open(&db_path).unwrap();
        (store, dir)
    }

    async fn call(tool: &CronjobTool, args: &str) -> ToolResult {
        let call = ToolCall::new(
            "1".into(),
            "cronjob".into(),
            serde_json::json!(serde_json::from_str::<serde_json::Value>(args).unwrap()),
        );
        tool.execute(call).await
    }

    fn sample_job(name: &str) -> CronJob {
        CronJob {
            id: None,
            name: name.to_string(),
            schedule: "every 30m".to_string(),
            task_prompt: "check status".to_string(),
            enabled: true,
            last_run: None,
            next_run: None,
        }
    }

    #[tokio::test]
    async fn create_and_list() {
        let (tool, _dir) = make_tool();
        let r = call(
            &tool,
            r#"{"action":"create","name":"test","schedule":"every 30m","task":"check status"}"#,
        )
        .await;
        assert!(!r.is_error);
        assert_eq!(r.output, "created job: test");
        let r = call(&tool, r#"{"action":"list"}"#).await;
        assert!(!r.is_error);
        assert!(r.output.contains("test"));
        assert!(r.output.contains("every 30m"));
        assert!(r.output.contains("enabled"));
    }

    #[tokio::test]
    async fn list_empty() {
        let (tool, _dir) = make_tool();
        let r = call(&tool, r#"{"action":"list"}"#).await;
        assert!(!r.is_error);
        assert_eq!(r.output, "(no cron jobs)");
    }

    #[tokio::test]
    async fn create_and_remove() {
        let (tool, _dir) = make_tool();
        call(
            &tool,
            r#"{"action":"create","name":"rm-me","schedule":"every 1h","task":"cleanup"}"#,
        )
        .await;
        let r = call(&tool, r#"{"action":"remove","name":"rm-me"}"#).await;
        assert!(!r.is_error);
        assert_eq!(r.output, "removed job: rm-me");
        let r = call(&tool, r#"{"action":"list"}"#).await;
        assert_eq!(r.output, "(no cron jobs)");
    }

    #[tokio::test]
    async fn update_schedule() {
        let (tool, _dir) = make_tool();
        call(
            &tool,
            r#"{"action":"create","name":"up","schedule":"every 30m","task":"task"}"#,
        )
        .await;
        let r = call(
            &tool,
            r#"{"action":"update","name":"up","schedule":"every 1h"}"#,
        )
        .await;
        assert!(!r.is_error);
        assert_eq!(r.output, "updated job: up");
        let r = call(&tool, r#"{"action":"list"}"#).await;
        assert!(r.output.contains("every 1h"));
    }

    #[tokio::test]
    async fn rejects_invalid_schedule() {
        let (tool, _dir) = make_tool();
        let r = call(
            &tool,
            r#"{"action":"create","name":"bad","schedule":"0 * * * *","task":"task"}"#,
        )
        .await;
        assert!(r.is_error);
        assert!(r.output.contains("invalid schedule"));
    }

    #[test]
    fn store_create_and_list() {
        let (store, _dir) = make_store();
        store.create_job(&sample_job("test-job")).unwrap();
        let jobs = store.list_jobs().unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, "test-job");
        assert_eq!(jobs[0].schedule, "every 30m");
        assert_eq!(jobs[0].task_prompt, "check status");
        assert!(jobs[0].enabled);
        assert!(jobs[0].next_run.is_some());
    }

    #[test]
    fn store_list_empty() {
        let (store, _dir) = make_store();
        let jobs = store.list_jobs().unwrap();
        assert!(jobs.is_empty());
    }

    #[test]
    fn store_create_and_remove() {
        let (store, _dir) = make_store();
        store.create_job(&sample_job("rm-me")).unwrap();
        store.remove_job_by_name("rm-me").unwrap();
        let jobs = store.list_jobs().unwrap();
        assert!(jobs.is_empty());
    }

    #[test]
    fn store_remove_nonexistent() {
        let (store, _dir) = make_store();
        let err = store.remove_job_by_name("nope").unwrap_err();
        assert!(err.contains("not found"));
    }

    #[test]
    fn store_update_schedule() {
        let (store, _dir) = make_store();
        store.create_job(&sample_job("up-me")).unwrap();
        store
            .update_job_by_name("up-me", Some("every 1h"), None, None)
            .unwrap();
        let jobs = store.list_jobs().unwrap();
        assert_eq!(jobs[0].schedule, "every 1h");
        assert_eq!(jobs[0].task_prompt, "check status");
    }

    #[test]
    fn store_update_task() {
        let (store, _dir) = make_store();
        store.create_job(&sample_job("up-task")).unwrap();
        store
            .update_job_by_name("up-task", None, Some("new task"), None)
            .unwrap();
        let jobs = store.list_jobs().unwrap();
        assert_eq!(jobs[0].task_prompt, "new task");
    }

    #[test]
    fn store_update_enabled() {
        let (store, _dir) = make_store();
        store.create_job(&sample_job("toggle")).unwrap();
        store
            .update_job_by_name("toggle", None, None, Some(false))
            .unwrap();
        let jobs = store.list_jobs().unwrap();
        assert!(!jobs[0].enabled);
    }

    #[test]
    fn store_get_due_jobs() {
        let (store, _dir) = make_store();
        let now = now_epoch();
        store.create_job(&sample_job("due-job")).unwrap();
        store
            .db
            .execute(
                "UPDATE cron_jobs SET next_run = ?1 WHERE name = 'due-job'",
                params![now - 10i64],
            )
            .unwrap();
        let due = store.get_due_jobs(now).unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].name, "due-job");
    }

    #[test]
    fn store_get_due_jobs_skips_disabled() {
        let (store, _dir) = make_store();
        let now = now_epoch();
        let mut job = sample_job("disabled-job");
        job.enabled = false;
        store.create_job(&job).unwrap();
        store
            .db
            .execute(
                "UPDATE cron_jobs SET next_run = ?1, enabled = 0 WHERE name = 'disabled-job'",
                params![now - 10i64],
            )
            .unwrap();
        let due = store.get_due_jobs(now).unwrap();
        assert!(due.is_empty());
    }

    #[test]
    fn store_mark_run() {
        let (store, _dir) = make_store();
        let id = store.create_job(&sample_job("mark-me")).unwrap();
        let now = now_epoch();
        let next = now + 1800;
        store.mark_run(id, now, next).unwrap();
        let jobs = store.list_jobs().unwrap();
        assert_eq!(jobs[0].last_run, Some(now));
        assert_eq!(jobs[0].next_run, Some(next));
    }

    #[test]
    fn store_find_by_name() {
        let (store, _dir) = make_store();
        store.create_job(&sample_job("find-me")).unwrap();
        let found = store.find_by_name("find-me").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "find-me");
        let missing = store.find_by_name("nope").unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn parse_interval_valid() {
        assert_eq!(parse_interval("every 30m").unwrap(), 1800);
        assert_eq!(parse_interval("every 1h").unwrap(), 3600);
        assert_eq!(parse_interval("every 24h").unwrap(), 86400);
        assert_eq!(parse_interval("every 60s").unwrap(), 60);
        assert_eq!(parse_interval("every 1d").unwrap(), 86400);
    }

    #[test]
    fn parse_interval_rejects_invalid() {
        assert!(parse_interval("0 * * * *").is_err());
        assert!(parse_interval("every day").is_err());
        assert!(parse_interval("hourly").is_err());
        assert!(parse_interval("").is_err());
    }
}
