use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubagentStatus {
    Spawned,
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

impl std::fmt::Display for SubagentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubagentStatus::Spawned => write!(f, "spawned"),
            SubagentStatus::Running => write!(f, "running"),
            SubagentStatus::Completed => write!(f, "completed"),
            SubagentStatus::Failed(e) => write!(f, "failed: {e}"),
            SubagentStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentResult {
    pub final_message: String,
    pub iterations: u32,
    pub tool_calls: u32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentHandle {
    pub id: String,
    pub task: String,
    pub status: SubagentStatus,
    pub started_at: SystemTime,
    pub completed_at: Option<SystemTime>,
    pub result: Option<SubagentResult>,
}

impl SubagentHandle {
    pub fn duration(&self) -> Option<std::time::Duration> {
        self.completed_at
            .map(|end| end.duration_since(self.started_at).unwrap_or_default())
    }
}

#[derive(Debug, Clone, Default)]
pub struct SubagentTracker {
    agents: Arc<Mutex<HashMap<String, SubagentHandle>>>,
}

impl SubagentTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&self, task: String) -> String {
        let id = format!("agent-{}", generate_id());
        let handle = SubagentHandle {
            id: id.clone(),
            task,
            status: SubagentStatus::Spawned,
            started_at: SystemTime::now(),
            completed_at: None,
            result: None,
        };
        if let Ok(mut agents) = self.agents.lock() {
            agents.insert(id.clone(), handle);
        }
        id
    }

    pub fn start(&self, id: &str) -> bool {
        if let Ok(mut agents) = self.agents.lock() {
            if let Some(handle) = agents.get_mut(id) {
                handle.status = SubagentStatus::Running;
                return true;
            }
        }
        false
    }

    pub fn complete(&self, id: &str, result: SubagentResult) -> bool {
        if let Ok(mut agents) = self.agents.lock() {
            if let Some(handle) = agents.get_mut(id) {
                handle.status = SubagentStatus::Completed;
                handle.completed_at = Some(SystemTime::now());
                handle.result = Some(result);
                return true;
            }
        }
        false
    }

    pub fn fail(&self, id: &str, error: String) -> bool {
        if let Ok(mut agents) = self.agents.lock() {
            if let Some(handle) = agents.get_mut(id) {
                handle.status = SubagentStatus::Failed(error);
                handle.completed_at = Some(SystemTime::now());
                return true;
            }
        }
        false
    }

    pub fn cancel(&self, id: &str) -> bool {
        if let Ok(mut agents) = self.agents.lock() {
            if let Some(handle) = agents.get_mut(id) {
                handle.status = SubagentStatus::Cancelled;
                handle.completed_at = Some(SystemTime::now());
                return true;
            }
        }
        false
    }

    pub fn list(&self) -> Vec<SubagentHandle> {
        match self.agents.lock() {
            Ok(agents) => agents.values().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    pub fn get(&self, id: &str) -> Option<SubagentHandle> {
        match self.agents.lock() {
            Ok(agents) => agents.get(id).cloned(),
            Err(_) => None,
        }
    }
}

fn generate_id() -> String {
    use std::fmt::Write;
    let mut buf = [0u8; 8];
    let _ = getrandom::fill(&mut buf);
    let mut s = String::with_capacity(16);
    for b in &buf {
        write!(s, "{b:02x}").unwrap();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_and_complete_lifecycle() {
        let tracker = SubagentTracker::new();
        let id = tracker.spawn("test task".to_string());

        let handle = tracker.get(&id).unwrap();
        assert_eq!(handle.status, SubagentStatus::Spawned);
        assert_eq!(handle.task, "test task");

        tracker.start(&id);
        let handle = tracker.get(&id).unwrap();
        assert_eq!(handle.status, SubagentStatus::Running);

        let result = SubagentResult {
            final_message: "done".to_string(),
            iterations: 3,
            tool_calls: 1,
            error: None,
        };
        tracker.complete(&id, result);

        let handle = tracker.get(&id).unwrap();
        assert_eq!(handle.status, SubagentStatus::Completed);
        assert!(handle.completed_at.is_some());
        assert_eq!(handle.result.as_ref().unwrap().final_message, "done");
        assert_eq!(handle.result.as_ref().unwrap().iterations, 3);
    }

    #[test]
    fn cancel_sets_status_to_cancelled() {
        let tracker = SubagentTracker::new();
        let id = tracker.spawn("cancel me".to_string());
        tracker.start(&id);

        let cancelled = tracker.cancel(&id);
        assert!(cancelled);

        let handle = tracker.get(&id).unwrap();
        assert_eq!(handle.status, SubagentStatus::Cancelled);
        assert!(handle.completed_at.is_some());
    }

    #[test]
    fn list_returns_all_agents() {
        let tracker = SubagentTracker::new();
        let id1 = tracker.spawn("task 1".to_string());
        let id2 = tracker.spawn("task 2".to_string());
        let id3 = tracker.spawn("task 3".to_string());

        let list = tracker.list();
        assert_eq!(list.len(), 3);

        let ids: Vec<&str> = list.iter().map(|h| h.id.as_str()).collect();
        assert!(ids.contains(&id1.as_str()));
        assert!(ids.contains(&id2.as_str()));
        assert!(ids.contains(&id3.as_str()));
    }

    #[test]
    fn fail_sets_error_status() {
        let tracker = SubagentTracker::new();
        let id = tracker.spawn("failing task".to_string());
        tracker.start(&id);

        let failed = tracker.fail(&id, "timeout exceeded".to_string());
        assert!(failed);

        let handle = tracker.get(&id).unwrap();
        assert_eq!(
            handle.status,
            SubagentStatus::Failed("timeout exceeded".to_string())
        );
        assert!(handle.completed_at.is_some());
    }

    #[test]
    fn get_returns_none_for_unknown_id() {
        let tracker = SubagentTracker::new();
        assert!(tracker.get("nonexistent").is_none());
    }

    #[test]
    fn operations_return_false_for_unknown_id() {
        let tracker = SubagentTracker::new();
        assert!(!tracker.start("unknown"));
        assert!(!tracker.complete(
            "unknown",
            SubagentResult {
                final_message: String::new(),
                iterations: 0,
                tool_calls: 0,
                error: None,
            }
        ));
        assert!(!tracker.fail("unknown", "err".to_string()));
        assert!(!tracker.cancel("unknown"));
    }

    #[test]
    fn status_display() {
        assert_eq!(SubagentStatus::Spawned.to_string(), "spawned");
        assert_eq!(SubagentStatus::Running.to_string(), "running");
        assert_eq!(SubagentStatus::Completed.to_string(), "completed");
        assert_eq!(
            SubagentStatus::Failed("oops".to_string()).to_string(),
            "failed: oops"
        );
        assert_eq!(SubagentStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn handle_duration_completed() {
        let tracker = SubagentTracker::new();
        let id = tracker.spawn("timed task".to_string());
        tracker.complete(
            &id,
            SubagentResult {
                final_message: "ok".to_string(),
                iterations: 1,
                tool_calls: 0,
                error: None,
            },
        );
        let handle = tracker.get(&id).unwrap();
        assert!(handle.duration().is_some());
    }

    #[test]
    fn handle_duration_none_when_not_completed() {
        let tracker = SubagentTracker::new();
        let id = tracker.spawn("still running".to_string());
        let handle = tracker.get(&id).unwrap();
        assert!(handle.duration().is_none());
    }

    #[test]
    fn spawn_generates_unique_ids() {
        let tracker = SubagentTracker::new();
        let id1 = tracker.spawn("a".to_string());
        let id2 = tracker.spawn("b".to_string());
        assert_ne!(id1, id2);
    }
}
