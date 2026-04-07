use std::collections::HashMap;

use crate::builtin::skill_trace::{SkillTrace, TraceOutcome, TraceStore};

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
        let traces = self.store.get_traces_for_skill(skill_name, 1000)?;
        let total_traces = traces.len();
        let success_rate = self.store.get_success_rate(skill_name, 1000)?;
        let recent_failures = self
            .store
            .count_recent_failures(skill_name, self.failure_window_hours)?;
        let needs_rewrite =
            success_rate < self.rewrite_threshold && total_traces >= self.min_traces;
        Ok(SkillHealth {
            skill_name: skill_name.to_string(),
            success_rate,
            total_traces,
            recent_failures,
            needs_rewrite,
        })
    }

    pub fn find_failure_patterns(&self, skill_name: &str) -> Result<Vec<FailurePattern>, String> {
        let traces = self.store.get_traces_for_skill(skill_name, 1000)?;
        let failures: Vec<&SkillTrace> = traces
            .iter()
            .filter(|t| t.outcome == TraceOutcome::Failure)
            .collect();

        if failures.is_empty() {
            return Ok(Vec::new());
        }

        let mut pattern_counts: HashMap<Vec<String>, usize> = HashMap::new();
        let mut pattern_examples: HashMap<Vec<String>, String> = HashMap::new();

        for trace in &failures {
            let tool_names: Vec<String> = trace.tool_chain.iter().map(|t| t.tool.clone()).collect();
            if tool_names.len() < 2 {
                continue;
            }
            for window_size in 2..=3 {
                if tool_names.len() < window_size {
                    continue;
                }
                for start in 0..=tool_names.len() - window_size {
                    let subseq: Vec<String> = tool_names[start..start + window_size].to_vec();
                    let count = pattern_counts.entry(subseq.clone()).or_insert(0);
                    *count += 1;
                    pattern_examples
                        .entry(subseq)
                        .or_insert_with(|| trace.task_summary.clone());
                }
            }
        }

        let mut patterns: Vec<FailurePattern> = pattern_counts
            .into_iter()
            .filter(|(_, count)| *count >= 2)
            .map(|(seq, count)| FailurePattern {
                example_task: pattern_examples.get(&seq).cloned().unwrap_or_default(),
                tool_sequence: seq,
                occurrence_count: count,
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
        let traces = self.store.get_traces_for_skill(skill_name, 1000)?;
        let mut successes = Vec::new();
        let mut failures = Vec::new();

        for trace in &traces {
            let summary = format_summary(trace);
            match trace.outcome {
                TraceOutcome::Success => {
                    if successes.len() < max_successes {
                        successes.push(summary);
                    }
                }
                TraceOutcome::Failure => {
                    if failures.len() < max_failures {
                        failures.push(summary);
                    }
                }
                TraceOutcome::Partial => {}
            }
        }

        Ok((successes, failures))
    }
}

fn format_summary(trace: &SkillTrace) -> String {
    let tools: Vec<&str> = trace.tool_chain.iter().map(|t| t.tool.as_str()).collect();
    let outcome_str = match trace.outcome {
        TraceOutcome::Success => "success",
        TraceOutcome::Failure => "failure",
        TraceOutcome::Partial => "partial",
    };
    format!(
        "[{}] {} | tools: {}",
        outcome_str,
        trace.task_summary,
        tools.join(" -> ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtin::skill_trace::ToolInvocation;
    use tempfile::TempDir;

    fn make_store() -> (TraceStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test-traces.sqlite3");
        let store = TraceStore::open(&db_path).unwrap();
        (store, dir)
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
        let (store, _dir) = make_store();
        for i in 0..5 {
            store
                .insert(&make_trace(
                    "deploy",
                    &format!("deploy {i}"),
                    TraceOutcome::Success,
                    &["terminal"],
                ))
                .unwrap();
        }
        let analyzer = TraceAnalyzer::new(store);
        let health = analyzer.analyze_skill("deploy").unwrap();
        assert!(!health.needs_rewrite);
        assert!((health.success_rate - 1.0).abs() < 0.01);
    }

    #[test]
    fn failing_skill_needs_rewrite() {
        let (store, _dir) = make_store();
        store
            .insert(&make_trace(
                "build",
                "ok build",
                TraceOutcome::Success,
                &["terminal"],
            ))
            .unwrap();
        for i in 0..3 {
            store
                .insert(&make_trace(
                    "build",
                    &format!("fail {i}"),
                    TraceOutcome::Failure,
                    &["terminal", "filesystem"],
                ))
                .unwrap();
        }
        let analyzer = TraceAnalyzer::new(store);
        let health = analyzer.analyze_skill("build").unwrap();
        assert!(health.needs_rewrite);
        assert!(health.success_rate < 0.5);
    }

    #[test]
    fn too_few_traces_no_rewrite() {
        let (store, _dir) = make_store();
        store
            .insert(&make_trace(
                "test",
                "only one",
                TraceOutcome::Failure,
                &["terminal"],
            ))
            .unwrap();
        let analyzer = TraceAnalyzer::new(store);
        let health = analyzer.analyze_skill("test").unwrap();
        assert!(!health.needs_rewrite);
    }

    #[test]
    fn find_failure_patterns_detects_repeated_sequence() {
        let (store, _dir) = make_store();
        let tools = &["read", "write", "exec"];
        for i in 0..3 {
            store
                .insert(&make_trace(
                    "build",
                    &format!("fail {i}"),
                    TraceOutcome::Failure,
                    tools,
                ))
                .unwrap();
        }
        let analyzer = TraceAnalyzer::new(store);
        let patterns = analyzer.find_failure_patterns("build").unwrap();
        assert!(!patterns.is_empty());
        assert!(patterns[0].occurrence_count >= 2);
    }

    #[test]
    fn gather_examples_separates_success_failure() {
        let (store, _dir) = make_store();
        store
            .insert(&make_trace(
                "deploy",
                "good deploy",
                TraceOutcome::Success,
                &["terminal"],
            ))
            .unwrap();
        store
            .insert(&make_trace(
                "deploy",
                "bad deploy",
                TraceOutcome::Failure,
                &["terminal"],
            ))
            .unwrap();
        let analyzer = TraceAnalyzer::new(store);
        let (successes, failures) = analyzer.gather_examples("deploy", 10, 10).unwrap();
        assert_eq!(successes.len(), 1);
        assert_eq!(failures.len(), 1);
        assert!(successes[0].contains("good deploy"));
        assert!(failures[0].contains("bad deploy"));
    }

    #[test]
    fn no_failures_yields_empty_patterns() {
        let (store, _dir) = make_store();
        for i in 0..3 {
            store
                .insert(&make_trace(
                    "deploy",
                    &format!("ok {i}"),
                    TraceOutcome::Success,
                    &["terminal"],
                ))
                .unwrap();
        }
        let analyzer = TraceAnalyzer::new(store);
        let patterns = analyzer.find_failure_patterns("deploy").unwrap();
        assert!(patterns.is_empty());
    }
}
