use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::builtin::skill_trace::{SkillTrace, TraceOutcome, TraceStore};
use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

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

pub type LlmRewriteFn = Box<
    dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send>> + Send + Sync,
>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteResult {
    pub skill_name: String,
    pub rewritten: bool,
    pub reason: String,
}

pub struct SkillRewriter {
    analyzer: TraceAnalyzer,
    skills_dir: PathBuf,
    llm_call: Arc<LlmRewriteFn>,
}

impl SkillRewriter {
    pub fn new(trace_store: TraceStore, skills_dir: PathBuf, llm_call: LlmRewriteFn) -> Self {
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
            "No failure examples.".to_string()
        } else {
            failures
                .iter()
                .enumerate()
                .map(|(i, f)| format!("Failure {}:\n{}", i + 1, f))
                .collect::<Vec<_>>()
                .join("\n\n")
        };

        let successes_text = if successes.is_empty() {
            "No success examples.".to_string()
        } else {
            successes
                .iter()
                .enumerate()
                .map(|(i, s)| format!("Success {}:\n{}", i + 1, s))
                .collect::<Vec<_>>()
                .join("\n\n")
        };

        let prompt = format!(
            r#"You are a skill improvement assistant. Rewrite the skill instructions to improve its success rate.

## Current Skill Instructions
```
{current_content}
```

## Failure Analysis
Success rate: {:.0}% over {} traces. Recent failures (24h): {}

## Failure Patterns
{patterns_text}

## Failure Examples
{failures_text}

## Success Examples
{successes_text}

Rewrite the skill instructions to:
1. Address the specific failure patterns
2. Preserve what works (from success examples)
3. Add guardrails for common failure points
4. Keep the skill concise and actionable

Output ONLY the improved skill content in Markdown."#,
            health.success_rate * 100.0,
            health.total_traces,
            health.recent_failures,
        );

        let rewritten = (self.llm_call)(prompt).await?;

        let skill_dir = self.skills_dir.join(skill_name);
        if skill_dir.exists() {
            let existing: Vec<String> = std::fs::read_dir(&skill_dir)
                .unwrap_or_else(|_| panic!("read_dir"))
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().to_str().map(String::from))
                .filter(|n| n.starts_with('v') && n.ends_with(".md"))
                .collect();
            let next_v = existing.len() + 1;
            let _ = std::fs::copy(&current_skill_path, skill_dir.join(format!("v{next_v}.md")));
        }

        std::fs::write(&current_skill_path, &rewritten)
            .map_err(|e| format!("failed to write rewritten skill: {e}"))?;

        Ok(RewriteResult {
            skill_name: skill_name.to_string(),
            rewritten: true,
            reason: format!(
                "rewrote skill (was {:.0}% success, {} recent failures)",
                health.success_rate * 100.0, health.recent_failures,
            ),
        })
    }
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
            .with_parameters(vec![ToolParameter::required(
                "skill_name",
                ParameterType::String,
                "Name of the skill to evolve",
            )]),
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

    #[tokio::test]
    async fn rewriter_skips_healthy_skill() {
        let (store, dir) = make_store();
        for i in 0..5 {
            store
                .insert(&make_trace(
                    "good",
                    &format!("ok {i}"),
                    TraceOutcome::Success,
                    &["terminal"],
                ))
                .unwrap();
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
        let (store, dir) = make_store();
        store
            .insert(&make_trace("bad", "ok", TraceOutcome::Success, &["terminal"]))
            .unwrap();
        for i in 0..3 {
            store
                .insert(&make_trace(
                    "bad",
                    &format!("fail {i}"),
                    TraceOutcome::Failure,
                    &["read_file", "terminal"],
                ))
                .unwrap();
        }
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(skills_dir.join("bad")).unwrap();
        std::fs::write(skills_dir.join("bad").join("SKILL.md"), "old instructions").unwrap();
        let rewriter = SkillRewriter::new(store, skills_dir.clone(), Box::new(|prompt| {
            Box::pin(async move {
                assert!(prompt.contains("old instructions"));
                assert!(prompt.contains("Failure"));
                Ok("improved instructions".to_string())
            })
        }));
        let result = rewriter.rewrite_skill("bad").await.unwrap();
        assert!(result.rewritten);
        let new_content =
            std::fs::read_to_string(skills_dir.join("bad").join("SKILL.md")).unwrap();
        assert_eq!(new_content, "improved instructions");
    }

    #[tokio::test]
    async fn skill_evolve_tool_end_to_end() {
        let (store, dir) = make_store();
        store
            .insert(&make_trace(
                "deploy",
                "deploy staging",
                TraceOutcome::Success,
                &["terminal"],
            ))
            .unwrap();
        for i in 0..3 {
            store
                .insert(&make_trace(
                    "deploy",
                    &format!("fail {i}"),
                    TraceOutcome::Failure,
                    &["read_file", "terminal"],
                ))
                .unwrap();
        }
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(skills_dir.join("deploy")).unwrap();
        std::fs::write(
            skills_dir.join("deploy").join("SKILL.md"),
            "# Deploy\nRun command",
        )
        .unwrap();
        let rewriter = Arc::new(SkillRewriter::new(
            store,
            skills_dir,
            Box::new(|_| {
                Box::pin(async {
                    Ok("# Deploy (v2)\nImproved with error handling".to_string())
                })
            }),
        ));
        let tool = SkillEvolveTool::new(rewriter);
        let call = ToolCall::new(
            "1".into(),
            "skill_evolve".into(),
            serde_json::json!({"skill_name": "deploy"}),
        );
        let result = tool.execute(call).await;
        assert!(!result.is_error);
        assert!(result.output.contains("evolved"));
    }
}
