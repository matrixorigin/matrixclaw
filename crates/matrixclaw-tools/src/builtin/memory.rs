use std::path::{Path, PathBuf};
use std::sync::Mutex;

use async_trait::async_trait;
use rusqlite::{params, Connection};

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct MemoryTool {
    descriptor: ToolDescriptor,
    db: Mutex<Connection>,
}

impl MemoryTool {
    pub fn open(db_path: &Path) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("memory: failed to create db directory: {e}"))?;
        }
        let conn =
            Connection::open(db_path).map_err(|e| format!("memory: failed to open db: {e}"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS memory (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )
        .map_err(|e| format!("memory: failed to create table: {e}"))?;

        Ok(Self {
            descriptor: ToolDescriptor::new(
                "memory",
                "Persistent key-value memory store. Data survives across sessions.",
            )
            .with_parameters(vec![
                ToolParameter::required("action", ParameterType::String, "Action to perform")
                    .enum_values(&["store", "retrieve", "list", "delete", "search"]),
                ToolParameter::optional("key", ParameterType::String, "Key for the memory entry"),
                ToolParameter::optional("value", ParameterType::String, "Value to store"),
                ToolParameter::optional(
                    "query",
                    ParameterType::String,
                    "Search query (substring match on key and value)",
                ),
            ]),
            db: Mutex::new(conn),
        })
    }

    pub fn db_path_for_home(home: &Path) -> PathBuf {
        home.join(".matrixclaw")
            .join("state")
            .join("memory.sqlite3")
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
impl ToolExecutor for MemoryTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let action = match call.arguments.get("action").and_then(|v| v.as_str()) {
            Some(a) => a,
            None => return ToolResult::error(&call, "missing required parameter: action"),
        };

        let db = match self.db.lock() {
            Ok(g) => g,
            Err(e) => return ToolResult::error(&call, format!("memory db lock: {e}")),
        };

        match action {
            "store" => {
                let key = match call.arguments.get("key").and_then(|v| v.as_str()) {
                    Some(k) => k,
                    None => return ToolResult::error(&call, "missing required parameter: key"),
                };
                let value = match call.arguments.get("value").and_then(|v| v.as_str()) {
                    Some(v) => v,
                    None => return ToolResult::error(&call, "missing required parameter: value"),
                };
                match db.execute(
                    "INSERT INTO memory (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = datetime('now')",
                    params![key, value],
                ) {
                    Ok(_) => ToolResult::success(&call, format!("stored {key}")),
                    Err(e) => ToolResult::error(&call, format!("store failed: {e}")),
                }
            }
            "retrieve" => {
                let key = match call.arguments.get("key").and_then(|v| v.as_str()) {
                    Some(k) => k,
                    None => return ToolResult::error(&call, "missing required parameter: key"),
                };
                let stmt = db.prepare("SELECT value FROM memory WHERE key = ?1");
                let mut stmt = match stmt {
                    Ok(s) => s,
                    Err(e) => return ToolResult::error(&call, format!("retrieve failed: {e}")),
                };
                match stmt.query_row(params![key], |row| row.get::<_, String>(0)) {
                    Ok(result) => ToolResult::success(&call, result),
                    Err(_) => ToolResult::error(&call, format!("key not found: {key}")),
                }
            }
            "list" => {
                let stmt = db.prepare("SELECT key FROM memory ORDER BY updated_at DESC");
                let mut stmt = match stmt {
                    Ok(s) => s,
                    Err(e) => return ToolResult::error(&call, format!("list failed: {e}")),
                };
                let rows = match stmt.query_map([], |row| row.get(0)) {
                    Ok(r) => r,
                    Err(e) => return ToolResult::error(&call, format!("list failed: {e}")),
                };
                let keys: Vec<String> = rows.filter_map(|r| r.ok()).collect();
                if keys.is_empty() {
                    ToolResult::success(&call, "(empty)")
                } else {
                    ToolResult::success(&call, keys.join("\n"))
                }
            }
            "delete" => {
                let key = match call.arguments.get("key").and_then(|v| v.as_str()) {
                    Some(k) => k,
                    None => return ToolResult::error(&call, "missing required parameter: key"),
                };
                match db.execute("DELETE FROM memory WHERE key = ?1", params![key]) {
                    Ok(0) => ToolResult::error(&call, format!("key not found: {key}")),
                    Ok(_) => ToolResult::success(&call, format!("deleted {key}")),
                    Err(e) => ToolResult::error(&call, format!("delete failed: {e}")),
                }
            }
            "search" => {
                let query = match call.arguments.get("query").and_then(|v| v.as_str()) {
                    Some(q) => q,
                    None => return ToolResult::error(&call, "missing required parameter: query"),
                };
                let pattern = format!("%{query}%");
                let stmt = db.prepare("SELECT key, value FROM memory WHERE key LIKE ?1 OR value LIKE ?1 ORDER BY updated_at DESC");
                let mut stmt = match stmt {
                    Ok(s) => s,
                    Err(e) => return ToolResult::error(&call, format!("search failed: {e}")),
                };
                let rows = match stmt.query_map(params![pattern], |row| {
                    let key: String = row.get(0)?;
                    let value: String = row.get(1)?;
                    Ok(format!("{key}: {value}"))
                }) {
                    Ok(r) => r,
                    Err(e) => return ToolResult::error(&call, format!("search failed: {e}")),
                };
                let results: Vec<String> = rows.filter_map(|r| r.ok()).collect();
                if results.is_empty() {
                    ToolResult::success(&call, "no matches found")
                } else {
                    ToolResult::success(&call, results.join("\n"))
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

    fn make_tool() -> (MemoryTool, TempDir) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test-memory.sqlite3");
        let tool = MemoryTool::open(&db_path).unwrap();
        (tool, dir)
    }

    async fn call(tool: &MemoryTool, args: &str) -> ToolResult {
        let call = ToolCall::new(
            "1".into(),
            "memory".into(),
            serde_json::json!(serde_json::from_str::<serde_json::Value>(args).unwrap()),
        );
        tool.execute(call).await
    }

    #[tokio::test]
    async fn store_and_retrieve() {
        let (tool, _dir) = make_tool();
        let r = call(&tool, r#"{"action":"store","key":"color","value":"blue"}"#).await;
        assert!(!r.is_error);
        let r = call(&tool, r#"{"action":"retrieve","key":"color"}"#).await;
        assert!(!r.is_error);
        assert_eq!(r.output, "blue");
    }

    #[tokio::test]
    async fn retrieve_missing_key() {
        let (tool, _dir) = make_tool();
        let r = call(&tool, r#"{"action":"retrieve","key":"nope"}"#).await;
        assert!(r.is_error);
        assert!(r.output.contains("not found"));
    }

    #[tokio::test]
    async fn list_empty() {
        let (tool, _dir) = make_tool();
        let r = call(&tool, r#"{"action":"list"}"#).await;
        assert!(!r.is_error);
        assert_eq!(r.output, "(empty)");
    }

    #[tokio::test]
    async fn delete_key() {
        let (tool, _dir) = make_tool();
        call(&tool, r#"{"action":"store","key":"x","value":"1"}"#).await;
        let r = call(&tool, r#"{"action":"delete","key":"x"}"#).await;
        assert!(!r.is_error);
        let r = call(&tool, r#"{"action":"retrieve","key":"x"}"#).await;
        assert!(r.is_error);
    }

    #[tokio::test]
    async fn search_finds_match() {
        let (tool, _dir) = make_tool();
        call(
            &tool,
            r#"{"action":"store","key":"fav-color","value":"blue"}"#,
        )
        .await;
        call(
            &tool,
            r#"{"action":"store","key":"fav-food","value":"sushi"}"#,
        )
        .await;
        let r = call(&tool, r#"{"action":"search","query":"blue"}"#).await;
        assert!(!r.is_error);
        assert!(r.output.contains("fav-color"));
        assert!(!r.output.contains("sushi"));
    }

    #[tokio::test]
    async fn search_no_match() {
        let (tool, _dir) = make_tool();
        call(&tool, r#"{"action":"store","key":"a","value":"hello"}"#).await;
        let r = call(&tool, r#"{"action":"search","query":"xyz"}"#).await;
        assert!(!r.is_error);
        assert_eq!(r.output, "no matches found");
    }

    #[tokio::test]
    async fn store_overwrites_existing() {
        let (tool, _dir) = make_tool();
        call(&tool, r#"{"action":"store","key":"k","value":"old"}"#).await;
        call(&tool, r#"{"action":"store","key":"k","value":"new"}"#).await;
        let r = call(&tool, r#"{"action":"retrieve","key":"k"}"#).await;
        assert_eq!(r.output, "new");
    }

    #[tokio::test]
    async fn persists_across_instances() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("persist.sqlite3");
        {
            let tool = MemoryTool::open(&db_path).unwrap();
            call(
                &tool,
                r#"{"action":"store","key":"persistent","value":"yes"}"#,
            )
            .await;
        }
        {
            let tool = MemoryTool::open(&db_path).unwrap();
            let r = call(&tool, r#"{"action":"retrieve","key":"persistent"}"#).await;
            assert!(!r.is_error);
            assert_eq!(r.output, "yes");
        }
    }
}
