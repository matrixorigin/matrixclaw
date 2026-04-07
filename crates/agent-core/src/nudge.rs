use matrixclaw_tools::builtin::nudge_store::NudgeStore;

pub struct NudgeEngine {
    store: Box<dyn NudgeStore>,
    threshold: f64,
    max_entries: usize,
}

impl NudgeEngine {
    pub fn new(store: Box<dyn NudgeStore>, threshold: f64, max_entries: usize) -> Self {
        Self {
            store,
            threshold,
            max_entries,
        }
    }

    pub fn nudge(&self, prompt: &str) -> Option<String> {
        let entries = self.store.search_relevant(prompt, self.max_entries);
        let relevant: Vec<_> = entries
            .into_iter()
            .filter(|e| e.relevance >= self.threshold)
            .collect();
        if relevant.is_empty() {
            return None;
        }
        let mut msg = "[Relevant context from past interactions]:\n".to_string();
        for entry in &relevant {
            msg.push_str(&format!("- {}: {}\n", entry.topic, entry.content));
        }
        Some(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use matrixclaw_tools::builtin::nudge_store::NudgeEntry;

    struct MockNudgeStore {
        entries: Vec<NudgeEntry>,
    }

    impl MockNudgeStore {
        fn new(entries: Vec<NudgeEntry>) -> Self {
            Self { entries }
        }
    }

    impl NudgeStore for MockNudgeStore {
        fn search_relevant(&self, _query: &str, limit: usize) -> Vec<NudgeEntry> {
            self.entries.iter().take(limit).cloned().collect()
        }
    }

    fn make_entry(topic: &str, content: &str, relevance: f64) -> NudgeEntry {
        NudgeEntry {
            topic: topic.to_string(),
            content: content.to_string(),
            relevance,
        }
    }

    #[test]
    fn nudge_returns_none_for_empty_store() {
        let store = MockNudgeStore::new(vec![]);
        let engine = NudgeEngine::new(Box::new(store), 0.5, 3);
        assert!(engine.nudge("test").is_none());
    }

    #[test]
    fn nudge_returns_entries_above_threshold() {
        let store = MockNudgeStore::new(vec![make_entry("deploy", "use kubectl", 0.8)]);
        let engine = NudgeEngine::new(Box::new(store), 0.6, 3);
        let result = engine.nudge("deploy app").unwrap();
        assert!(result.contains("deploy: use kubectl"));
    }

    #[test]
    fn nudge_skips_entries_below_threshold() {
        let store = MockNudgeStore::new(vec![make_entry("a", "alpha", 0.3)]);
        let engine = NudgeEngine::new(Box::new(store), 0.6, 3);
        assert!(engine.nudge("test").is_none());
    }

    #[test]
    fn nudge_formats_context_correctly() {
        let store = MockNudgeStore::new(vec![
            make_entry("deploy", "use kubectl", 0.9),
            make_entry("test", "use cargo test", 0.8),
        ]);
        let engine = NudgeEngine::new(Box::new(store), 0.5, 3);
        let result = engine.nudge("deploy").unwrap();
        assert!(result.starts_with("[Relevant context from past interactions]:"));
        assert!(result.contains("- deploy: use kubectl"));
        assert!(result.contains("- test: use cargo test"));
    }

    #[test]
    fn nudge_limits_to_max_entries() {
        let store = MockNudgeStore::new(vec![
            make_entry("a", "alpha", 0.9),
            make_entry("b", "bravo", 0.8),
            make_entry("c", "charlie", 0.7),
        ]);
        let engine = NudgeEngine::new(Box::new(store), 0.5, 2);
        let result = engine.nudge("test").unwrap();
        assert!(result.contains("- a: alpha"));
        assert!(result.contains("- b: bravo"));
        assert!(!result.contains("- c: charlie"));
    }
}
