use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::{params, Connection};

pub trait NudgeStore: Send + Sync {
    fn search_relevant(&self, query: &str, limit: usize) -> Vec<NudgeEntry>;
}

pub struct NudgeEntry {
    pub topic: String,
    pub content: String,
    pub relevance: f64,
}

impl Clone for NudgeEntry {
    fn clone(&self) -> Self {
        Self {
            topic: self.topic.clone(),
            content: self.content.clone(),
            relevance: self.relevance,
        }
    }
}

pub struct MemoryNudgeStore {
    db: Mutex<Connection>,
}

impl MemoryNudgeStore {
    pub fn open(db_path: &Path) -> Result<Self, String> {
        if !db_path.exists() {
            return Err("memory database not found".to_string());
        }
        let conn =
            Connection::open(db_path).map_err(|e| format!("nudge: failed to open db: {e}"))?;
        Ok(Self {
            db: Mutex::new(conn),
        })
    }

    pub fn db_path_for_home(home: &Path) -> PathBuf {
        home.join(".matrixclaw")
            .join("state")
            .join("memory.sqlite3")
    }
}

impl NudgeStore for MemoryNudgeStore {
    fn search_relevant(&self, query: &str, limit: usize) -> Vec<NudgeEntry> {
        let db = match self.db.lock() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };

        let keywords: Vec<&str> = query
            .split_whitespace()
            .filter(|w| w.len() >= 3)
            .take(5)
            .collect();

        if keywords.is_empty() {
            return Vec::new();
        }

        let mut best: HashMap<String, NudgeEntry> = HashMap::new();

        for keyword in &keywords {
            let pattern = format!("%{keyword}%");
            let mut stmt = match db.prepare(
                "SELECT key, value FROM memory WHERE key LIKE ?1 OR value LIKE ?1 ORDER BY updated_at DESC"
            ) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let rows = match stmt.query_map(params![pattern], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            }) {
                Ok(r) => r,
                Err(_) => continue,
            };

            let kw_lower = keyword.to_lowercase();
            for row in rows {
                let (key, value) = match row {
                    Ok(r) => r,
                    Err(_) => continue,
                };

                let relevance = if key.to_lowercase().contains(&kw_lower) {
                    1.0
                } else {
                    0.7
                };

                best.entry(key.clone())
                    .and_modify(|e| {
                        if relevance > e.relevance {
                            e.relevance = relevance;
                        }
                    })
                    .or_insert(NudgeEntry {
                        topic: key,
                        content: value,
                        relevance,
                    });
            }
        }

        let mut entries: Vec<NudgeEntry> = best.into_values().collect();
        entries.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        entries.truncate(limit);
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_db(dir: &Path) -> PathBuf {
        let db_path = dir.join("test-memory.sqlite3");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS memory (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )
        .unwrap();
        drop(conn);
        db_path
    }

    fn seed(db_path: &Path, entries: &[(&str, &str)]) {
        let conn = Connection::open(db_path).unwrap();
        for (key, value) in entries {
            conn.execute(
                "INSERT INTO memory (key, value) VALUES (?1, ?2)",
                params![key, value],
            )
            .unwrap();
        }
    }

    fn open_store(db_path: &Path) -> MemoryNudgeStore {
        MemoryNudgeStore::open(db_path).unwrap()
    }

    #[test]
    fn search_finds_by_key_match() {
        let dir = TempDir::new().unwrap();
        let db_path = setup_db(dir.path());
        seed(&db_path, &[("deployment-strategy", "use blue-green")]);
        let store = open_store(&db_path);
        let results = store.search_relevant("deployment", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].topic, "deployment-strategy");
        assert_eq!(results[0].relevance, 1.0);
    }

    #[test]
    fn search_finds_by_value_match() {
        let dir = TempDir::new().unwrap();
        let db_path = setup_db(dir.path());
        seed(&db_path, &[("notes", "deploy using helm charts")]);
        let store = open_store(&db_path);
        let results = store.search_relevant("deploy", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].topic, "notes");
        assert_eq!(results[0].relevance, 0.7);
    }

    #[test]
    fn search_returns_empty_for_no_matches() {
        let dir = TempDir::new().unwrap();
        let db_path = setup_db(dir.path());
        seed(&db_path, &[("color", "blue")]);
        let store = open_store(&db_path);
        let results = store.search_relevant("xyz", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn relevance_scoring() {
        let dir = TempDir::new().unwrap();
        let db_path = setup_db(dir.path());
        seed(
            &db_path,
            &[
                ("deploy-steps", "how to deploy"),
                ("random-key", "deploy is mentioned here"),
            ],
        );
        let store = open_store(&db_path);
        let results = store.search_relevant("deploy", 10);
        assert_eq!(results.len(), 2);
        let key_match: Vec<_> = results.iter().filter(|e| e.relevance == 1.0).collect();
        let value_match: Vec<_> = results.iter().filter(|e| e.relevance == 0.7).collect();
        assert_eq!(key_match.len(), 1);
        assert_eq!(value_match.len(), 1);
    }
}
