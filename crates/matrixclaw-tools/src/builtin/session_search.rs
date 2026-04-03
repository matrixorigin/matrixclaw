use std::path::{Path, PathBuf};
use std::sync::Mutex;

use async_trait::async_trait;
use rusqlite::{params, Connection};

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct SessionSearchTool {
    descriptor: ToolDescriptor,
    db: Mutex<Connection>,
}

impl SessionSearchTool {
    pub fn open(db_path: &Path) -> Result<Self, String> {
        let conn = Connection::open(db_path)
            .map_err(|e| format!("session search: failed to open db: {e}"))?;
        Ok(Self {
            descriptor: ToolDescriptor::new(
                "session_search",
                "Search past conversation history. Returns matching messages with context snippets.",
            )
            .with_parameters(vec![
                ToolParameter::required("query", ParameterType::String, "Search query"),
                ToolParameter::optional("limit", ParameterType::String, "Maximum results (default 10)"),
            ]),
            db: Mutex::new(conn),
        })
    }

    pub fn db_path_for_home(home: &Path) -> PathBuf {
        home.join(".matrixclaw").join("state").join("sessions.db")
    }
}

#[async_trait]
impl ToolExecutor for SessionSearchTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let query = match call.arguments.get("query").and_then(|v| v.as_str()) {
            Some(q) => q,
            None => return ToolResult::error(&call, "missing required parameter: query"),
        };
        let limit_str = call
            .arguments
            .get("limit")
            .and_then(|v| v.as_str())
            .unwrap_or("10");
        let limit: usize = limit_str.parse().unwrap_or(10);

        let db = match self.db.lock() {
            Ok(g) => g,
            Err(e) => return ToolResult::error(&call, format!("session search db lock: {e}")),
        };

        let fts_query = query.replace('"', "\"\"");
        let mut stmt = match db.prepare(
            "SELECT m.kind, snippet(messages_fts, -1, '<<', '>>', '...', 32) as snippet FROM messages_fts f JOIN transcript m ON m.id = f.rowid WHERE messages_fts MATCH ?1 ORDER BY rank LIMIT ?2"
        ) {
            Ok(s) => s,
            Err(e) => return ToolResult::error(&call, format!("search query failed: {e}")),
        };

        let results: Vec<String> = match stmt.query_map(params![fts_query, limit], |row| {
            let kind: String = row.get(0)?;
            let snippet: String = row.get::<_, Option<String>>(1)?.unwrap_or_default();
            Ok(format!("[{kind}] {snippet}"))
        }) {
            Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
            Err(e) => return ToolResult::error(&call, format!("search failed: {e}")),
        };

        if results.is_empty() {
            ToolResult::success(&call, "no matches found")
        } else {
            ToolResult::success(&call, results.join("\n\n"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (SessionSearchTool, TempDir) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test-sessions.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS transcript (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                kind TEXT NOT NULL,
                content TEXT NOT NULL
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
                content,
                content=transcript,
                content_rowid=id
            );
            CREATE TRIGGER IF NOT EXISTS messages_ai AFTER INSERT ON transcript BEGIN
                INSERT INTO messages_fts(rowid, content) VALUES (new.id, new.content);
            END;",
        )
        .unwrap();
        drop(conn);
        let tool = SessionSearchTool::open(&db_path).unwrap();
        (tool, dir)
    }

    fn insert_message(tool: &SessionSearchTool, kind: &str, content: &str) {
        let db = tool.db.lock().unwrap();
        db.execute(
            "INSERT INTO transcript (kind, content) VALUES (?1, ?2)",
            params![kind, content],
        )
        .unwrap();
    }

    async fn call(tool: &SessionSearchTool, args: &str) -> ToolResult {
        let call = ToolCall::new(
            "1".into(),
            "session_search".into(),
            serde_json::json!(serde_json::from_str::<serde_json::Value>(args).unwrap()),
        );
        tool.execute(call).await
    }

    #[tokio::test]
    async fn search_finds_match() {
        let (tool, _dir) = setup();
        insert_message(&tool, "user", "how do I fix the docker build?");
        insert_message(&tool, "assistant", "try running docker build --no-cache");
        let r = call(&tool, r#"{"query": "docker build"}"#).await;
        assert!(!r.is_error);
        assert!(r.output.contains("docker"));
    }

    #[tokio::test]
    async fn search_no_match() {
        let (tool, _dir) = setup();
        insert_message(&tool, "user", "hello world");
        let r = call(&tool, r#"{"query": "quantum computing"}"#).await;
        assert!(!r.is_error);
        assert!(r.output.contains("no matches"));
    }

    #[tokio::test]
    async fn search_requires_query() {
        let (tool, _dir) = setup();
        let r = call(&tool, r#"{}"#).await;
        assert!(r.is_error);
        assert!(r.output.contains("query"));
    }
}
