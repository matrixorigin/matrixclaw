use rusqlite::{params, Connection};
use std::path::Path;

pub struct CostTracker {
    conn: Connection,
}

impl CostTracker {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create cost db directory: {e}"))?;
        }
        let conn =
            Connection::open(path).map_err(|e| format!("failed to open cost database: {e}"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS cost_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                model TEXT NOT NULL,
                cost_usd REAL NOT NULL,
                recorded_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_cost_session ON cost_entries(session_id);
            CREATE INDEX IF NOT EXISTS idx_cost_model ON cost_entries(model);",
        )
        .map_err(|e| format!("failed to initialize cost schema: {e}"))?;
        Ok(Self { conn })
    }

    pub fn record_cost(&self, session_id: &str, model: &str, cost_usd: f64) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO cost_entries (session_id, model, cost_usd) VALUES (?1, ?2, ?3)",
                params![session_id, model, cost_usd],
            )
            .map_err(|e| format!("failed to record cost: {e}"))?;
        Ok(())
    }

    pub fn session_total(&self, session_id: &str) -> Result<f64, String> {
        let total: f64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(cost_usd), 0.0) FROM cost_entries WHERE session_id = ?1",
                params![session_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("failed to query session cost: {e}"))?;
        Ok(total)
    }

    pub fn model_total(&self, model: &str) -> Result<f64, String> {
        let total: f64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(cost_usd), 0.0) FROM cost_entries WHERE model = ?1",
                params![model],
                |row| row.get(0),
            )
            .map_err(|e| format!("failed to query model cost: {e}"))?;
        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_and_queries_cost_db() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("costs.sqlite3");
        let tracker = CostTracker::open(&db_path).unwrap();

        tracker.record_cost("session-1", "gpt-4o", 0.015).unwrap();
        tracker.record_cost("session-1", "gpt-4o", 0.008).unwrap();
        tracker
            .record_cost("session-1", "claude-sonnet-4-20250514", 0.025)
            .unwrap();

        let session_total = tracker.session_total("session-1").unwrap();
        assert!((session_total - 0.048).abs() < 0.0001);

        let model_total = tracker.model_total("gpt-4o").unwrap();
        assert!((model_total - 0.023).abs() < 0.0001);
    }

    #[test]
    fn empty_session_returns_zero() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("costs.sqlite3");
        let tracker = CostTracker::open(&db_path).unwrap();
        assert_eq!(tracker.session_total("nonexistent").unwrap(), 0.0);
    }
}
