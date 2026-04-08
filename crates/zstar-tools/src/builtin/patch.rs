use std::path::PathBuf;

use async_trait::async_trait;

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

fn validate_path(workspace_root: &str, path: &str) -> Result<PathBuf, String> {
    if path.contains("..") || path.starts_with('/') {
        return Err(
            "path traversal detected: path must be relative and cannot contain '..'".to_string(),
        );
    }
    let workspace = std::path::Path::new(workspace_root)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(workspace_root));
    let target = workspace.join(path);
    match target.canonicalize() {
        Ok(canonical) => {
            if canonical.starts_with(&workspace) {
                Ok(canonical)
            } else {
                Err("path traversal detected: path is outside workspace".to_string())
            }
        }
        Err(_) => {
            if let Some(parent) = target.parent() {
                if parent.exists() {
                    let canonical_parent = parent
                        .canonicalize()
                        .unwrap_or_else(|_| parent.to_path_buf());
                    if canonical_parent.starts_with(&workspace) {
                        Ok(target)
                    } else {
                        Err("path traversal detected: path is outside workspace".to_string())
                    }
                } else {
                    Ok(target)
                }
            } else {
                Ok(target)
            }
        }
    }
}

fn normalize_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_was_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
        } else {
            result.push(ch);
            prev_was_space = false;
        }
    }
    result.trim().to_string()
}

struct Match {
    start: usize,
    end: usize,
}

fn find_exact(content: &str, old: &str) -> Vec<Match> {
    let mut matches = Vec::new();
    let mut start = 0;
    while let Some(pos) = content[start..].find(old) {
        let abs_pos = start + pos;
        matches.push(Match {
            start: abs_pos,
            end: abs_pos + old.len(),
        });
        start = abs_pos + 1;
    }
    matches
}

struct LineInfo {
    lines: Vec<String>,
    offsets: Vec<usize>,
}

fn split_lines(content: &str) -> LineInfo {
    let mut lines = Vec::new();
    let mut offsets = Vec::new();
    let mut pos = 0;
    for line in content.split_inclusive('\n') {
        offsets.push(pos);
        lines.push(line.to_string());
        pos += line.len();
    }
    LineInfo { lines, offsets }
}

fn line_content_end_byte(info: &LineInfo, line_idx: usize) -> usize {
    let line = &info.lines[line_idx];
    info.offsets[line_idx] + line.trim_end_matches('\n').len()
}

fn find_prefix_lines(content: &str, old: &str) -> Vec<Match> {
    let old_lines: Vec<&str> = old.lines().collect();
    if old_lines.is_empty() {
        return Vec::new();
    }
    let first_line = old_lines[0];
    let info = split_lines(content);
    let mut matches = Vec::new();

    for i in 0..info.lines.len() {
        if info.lines[i].trim() == first_line.trim() {
            if i + old_lines.len() > info.lines.len() {
                continue;
            }
            let mut all_match = true;
            for (j, old_line) in old_lines.iter().enumerate() {
                if info.lines[i + j].trim() != old_line.trim() {
                    all_match = false;
                    break;
                }
            }
            if all_match {
                let start_byte = info.offsets[i];
                let last_idx = i + old_lines.len() - 1;
                let end_byte = line_content_end_byte(&info, last_idx);
                matches.push(Match {
                    start: start_byte,
                    end: end_byte,
                });
            }
        }
    }
    matches
}

fn find_suffix_lines(content: &str, old: &str) -> Vec<Match> {
    let old_lines: Vec<&str> = old.lines().collect();
    if old_lines.is_empty() {
        return Vec::new();
    }
    let last_line = old_lines[old_lines.len() - 1];
    let info = split_lines(content);
    let mut matches = Vec::new();

    for i in 0..info.lines.len() {
        if info.lines[i].trim() == last_line.trim() {
            if i + 1 < old_lines.len() {
                continue;
            }
            let start_idx = i + 1 - old_lines.len();
            let mut all_match = true;
            for (j, old_line) in old_lines.iter().enumerate() {
                if info.lines[start_idx + j].trim() != old_line.trim() {
                    all_match = false;
                    break;
                }
            }
            if all_match {
                let start_byte = info.offsets[start_idx];
                let end_byte = line_content_end_byte(&info, i);
                matches.push(Match {
                    start: start_byte,
                    end: end_byte,
                });
            }
        }
    }
    matches
}

fn find_contains(content: &str, old: &str) -> Vec<Match> {
    let norm_old = normalize_whitespace(old);
    if norm_old.is_empty() {
        return Vec::new();
    }
    let info = split_lines(content);
    let old_line_count = old.lines().count().max(1);
    let max_window = old_line_count * 3;
    let mut byte_matches = Vec::new();

    for i in 0..info.lines.len() {
        let upper = max_window.min(info.lines.len() - i);
        for w in old_line_count..=upper {
            let window: String = info.lines[i..i + w].concat();
            if normalize_whitespace(&window) == norm_old {
                let start_byte = info.offsets[i];
                let last_idx = i + w - 1;
                let end_byte = line_content_end_byte(&info, last_idx);
                byte_matches.push(Match {
                    start: start_byte,
                    end: end_byte,
                });
                break;
            }
        }
    }
    byte_matches
}

fn find_fuzzy(content: &str, old: &str) -> Vec<Match> {
    let old_lines: Vec<&str> = old.lines().collect();
    let info = split_lines(content);
    if old_lines.is_empty() {
        return Vec::new();
    }
    let n = old_lines.len();
    if info.lines.len() < n {
        return Vec::new();
    }

    let mut best_score: f64 = 0.0;
    let mut best_idx: Option<usize> = None;

    for i in 0..=info.lines.len() - n {
        let mut matching = 0usize;
        for (j, old_line) in old_lines.iter().enumerate() {
            let file_line = &info.lines[i + j];
            let file_trimmed = file_line.trim_end();
            let old_trimmed = old_line.trim_end();
            if file_trimmed == old_trimmed {
                matching += 1;
            } else {
                let norm_file = normalize_whitespace(file_line);
                let norm_old = normalize_whitespace(old_line);
                if norm_file == norm_old || line_similarity(&norm_file, &norm_old) >= 0.7 {
                    matching += 1;
                }
            }
        }
        let score = matching as f64 / n as f64;
        if score >= 0.7 && score > best_score {
            best_score = score;
            best_idx = Some(i);
        }
    }

    match best_idx {
        Some(idx) => {
            let start_byte = info.offsets[idx];
            let last_idx = idx + n - 1;
            let end_byte = line_content_end_byte(&info, last_idx);
            vec![Match {
                start: start_byte,
                end: end_byte,
            }]
        }
        None => Vec::new(),
    }
}

fn line_similarity(a: &str, b: &str) -> f64 {
    if a == b {
        return 1.0;
    }
    let len_a = a.len();
    let len_b = b.len();
    if len_a == 0 && len_b == 0 {
        return 1.0;
    }
    if len_a == 0 || len_b == 0 {
        return 0.0;
    }
    let max_len = len_a.max(len_b);
    if max_len == 0 {
        return 1.0;
    }
    let dist = levenshtein_distance(a, b);
    1.0 - (dist as f64 / max_len as f64)
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut prev: Vec<usize> = (0..=b_len).collect();
    let mut curr: Vec<usize> = vec![0; b_len + 1];

    for i in 1..=a_len {
        curr[0] = i;
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b_len]
}

fn find_regex(content: &str, pattern: &str) -> Result<Vec<Match>, String> {
    let re = regex::Regex::new(pattern).map_err(|e| format!("invalid regex: {e}"))?;
    let matches: Vec<Match> = re
        .find_iter(content)
        .map(|m| Match {
            start: m.start(),
            end: m.end(),
        })
        .collect();
    Ok(matches)
}

pub struct PatchTool {
    descriptor: ToolDescriptor,
    workspace_root: String,
}

impl PatchTool {
    pub fn new(workspace_root: &str) -> Self {
        let mut strategy_param = ToolParameter::optional(
            "strategy",
            ParameterType::String,
            "Matching strategy: exact, prefix_lines, suffix_lines, contains, fuzzy, regex. Default: exact",
        );
        strategy_param.enum_values = Some(vec![
            "exact".to_string(),
            "prefix_lines".to_string(),
            "suffix_lines".to_string(),
            "contains".to_string(),
            "fuzzy".to_string(),
            "regex".to_string(),
        ]);

        Self {
            descriptor: ToolDescriptor::new(
                "patch",
                "Apply a diff/patch to a file with fuzzy matching. Supports exact, prefix, suffix, and contains matching strategies. More flexible than edit_file for whitespace and formatting differences.",
            )
            .with_parameters(vec![
                ToolParameter::required(
                    "path",
                    ParameterType::String,
                    "File path to patch (relative to workspace)",
                ),
                ToolParameter::required(
                    "old",
                    ParameterType::String,
                    "The text to find in the file",
                ),
                ToolParameter::required(
                    "new",
                    ParameterType::String,
                    "The replacement text",
                ),
                strategy_param,
                ToolParameter::optional(
                    "allow_multiple",
                    ParameterType::Boolean,
                    "Allow replacing all matches, not just the first. Default: false",
                ),
            ]),
            workspace_root: workspace_root.to_string(),
        }
    }
}

#[async_trait]
impl ToolExecutor for PatchTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let path = match call.arguments.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult::error(&call, "missing required parameter: path"),
        };

        let old = match call.arguments.get("old").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => return ToolResult::error(&call, "missing required parameter: old"),
        };

        let new = match call.arguments.get("new").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => return ToolResult::error(&call, "missing required parameter: new"),
        };

        let strategy = call
            .arguments
            .get("strategy")
            .and_then(|v| v.as_str())
            .unwrap_or("exact");

        let allow_multiple = call
            .arguments
            .get("allow_multiple")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let validated = match validate_path(&self.workspace_root, path) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(&call, e),
        };

        let content = match tokio::fs::read_to_string(&validated).await {
            Ok(c) => c,
            Err(e) => return ToolResult::error(&call, format!("failed to read file: {e}")),
        };

        let matches = match strategy {
            "exact" => find_exact(&content, old),
            "prefix_lines" => find_prefix_lines(&content, old),
            "suffix_lines" => find_suffix_lines(&content, old),
            "contains" => find_contains(&content, old),
            "fuzzy" => find_fuzzy(&content, old),
            "regex" => match find_regex(&content, old) {
                Ok(m) => m,
                Err(e) => return ToolResult::error(&call, e),
            },
            other => {
                return ToolResult::error(
                    &call,
                    format!("unknown strategy: {other}. Use: exact, prefix_lines, suffix_lines, contains, fuzzy, regex"),
                )
            }
        };

        if matches.is_empty() {
            return ToolResult::error(
                &call,
                format!("no match found using {strategy} strategy for the given old text"),
            );
        }

        if matches.len() > 1 && !allow_multiple {
            return ToolResult::error(
                &call,
                format!(
                    "found {} matches but allow_multiple is false. Set allow_multiple to true to replace all occurrences.",
                    matches.len()
                ),
            );
        }

        let mut new_content = String::with_capacity(content.len());
        let mut last_end = 0;
        for m in &matches {
            new_content.push_str(&content[last_end..m.start]);
            new_content.push_str(new);
            last_end = m.end;
        }
        new_content.push_str(&content[last_end..]);

        match tokio::fs::write(&validated, &new_content).await {
            Ok(_) => ToolResult::success(
                &call,
                format!(
                    "patched {} occurrence(s) in {} using {strategy} strategy",
                    matches.len(),
                    path
                ),
            ),
            Err(e) => ToolResult::error(&call, format!("failed to write file: {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> (PatchTool, TempDir) {
        let dir = TempDir::new().unwrap();
        let tool = PatchTool::new(dir.path().to_str().unwrap());
        (tool, dir)
    }

    async fn call_tool(tool: &PatchTool, args: serde_json::Value) -> ToolResult {
        let call = ToolCall::new("1".into(), "patch".into(), args);
        tool.execute(call).await
    }

    #[tokio::test]
    async fn patch_exact_match() {
        let (tool, dir) = setup();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "hello world\nfoo bar\nbaz\n").unwrap();
        let result = call_tool(
            &tool,
            serde_json::json!({
                "path": "test.txt",
                "old": "foo bar",
                "new": "foo baz",
                "strategy": "exact"
            }),
        )
        .await;
        assert!(!result.is_error, "error: {}", result.output);
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "hello world\nfoo baz\nbaz\n");
    }

    #[tokio::test]
    async fn patch_exact_no_match() {
        let (tool, dir) = setup();
        fs::write(dir.path().join("test.txt"), "hello world\n").unwrap();
        let result = call_tool(
            &tool,
            serde_json::json!({
                "path": "test.txt",
                "old": "not found",
                "new": "replacement",
                "strategy": "exact"
            }),
        )
        .await;
        assert!(result.is_error);
        assert!(result.output.contains("no match found"));
    }

    #[tokio::test]
    async fn patch_contains_normalized_whitespace() {
        let (tool, dir) = setup();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "hello   world\n\nfoo    bar\n").unwrap();
        let result = call_tool(
            &tool,
            serde_json::json!({
                "path": "test.txt",
                "old": "hello world\nfoo bar",
                "new": "replaced",
                "strategy": "contains"
            }),
        )
        .await;
        assert!(!result.is_error, "error: {}", result.output);
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("replaced"));
        assert!(!content.contains("hello"));
    }

    #[tokio::test]
    async fn patch_prefix_lines() {
        let (tool, dir) = setup();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "  first line  \nsecond line\nthird line\n").unwrap();
        let result = call_tool(
            &tool,
            serde_json::json!({
                "path": "test.txt",
                "old": "first line\nsecond line",
                "new": "REPLACED",
                "strategy": "prefix_lines"
            }),
        )
        .await;
        assert!(!result.is_error, "error: {}", result.output);
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "REPLACED\nthird line\n");
    }

    #[tokio::test]
    async fn patch_suffix_lines() {
        let (tool, dir) = setup();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "first line\nsecond line\n  third line  \n").unwrap();
        let result = call_tool(
            &tool,
            serde_json::json!({
                "path": "test.txt",
                "old": "second line\nthird line",
                "new": "REPLACED",
                "strategy": "suffix_lines"
            }),
        )
        .await;
        assert!(!result.is_error, "error: {}", result.output);
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "first line\nREPLACED\n");
    }

    #[tokio::test]
    async fn patch_fuzzy_similarity() {
        let (tool, dir) = setup();
        let file_path = dir.path().join("test.txt");
        fs::write(
            &file_path,
            "fn main() {\n    println!(\"hello\");\n}\n\nfn helper() {\n    todo!()\n}\n",
        )
        .unwrap();
        let result = call_tool(
            &tool,
            serde_json::json!({
                "path": "test.txt",
                "old": "fn main() {\n    println!(\"world\");\n}",
                "new": "fn main() {\n    println!(\"replaced\");\n}",
                "strategy": "fuzzy"
            }),
        )
        .await;
        assert!(!result.is_error, "error: {}", result.output);
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("replaced"));
        assert!(!content.contains("hello"));
    }

    #[tokio::test]
    async fn patch_fuzzy_below_threshold() {
        let (tool, dir) = setup();
        fs::write(
            dir.path().join("test.txt"),
            "completely different content here\nnothing matches\n",
        )
        .unwrap();
        let result = call_tool(
            &tool,
            serde_json::json!({
                "path": "test.txt",
                "old": "fn main() {\n    println!(\"hello\");\n}",
                "new": "replaced",
                "strategy": "fuzzy"
            }),
        )
        .await;
        assert!(result.is_error);
        assert!(result.output.contains("no match found"));
    }

    #[tokio::test]
    async fn patch_regex_match() {
        let (tool, dir) = setup();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "foo123bar\nfoo456bar\n").unwrap();
        let result = call_tool(
            &tool,
            serde_json::json!({
                "path": "test.txt",
                "old": "foo\\d+bar",
                "new": "MATCHED",
                "strategy": "regex",
                "allow_multiple": true
            }),
        )
        .await;
        assert!(!result.is_error, "error: {}", result.output);
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "MATCHED\nMATCHED\n");
    }

    #[tokio::test]
    async fn patch_multiple_matches_blocked() {
        let (tool, dir) = setup();
        fs::write(dir.path().join("test.txt"), "foo\nbar\nfoo\nbaz\n").unwrap();
        let result = call_tool(
            &tool,
            serde_json::json!({
                "path": "test.txt",
                "old": "foo",
                "new": "replaced",
                "strategy": "exact"
            }),
        )
        .await;
        assert!(result.is_error);
        assert!(result.output.contains("found 2 matches"));
    }

    #[tokio::test]
    async fn patch_multiple_matches_allowed() {
        let (tool, dir) = setup();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "foo\nbar\nfoo\nbaz\n").unwrap();
        let result = call_tool(
            &tool,
            serde_json::json!({
                "path": "test.txt",
                "old": "foo",
                "new": "replaced",
                "strategy": "exact",
                "allow_multiple": true
            }),
        )
        .await;
        assert!(!result.is_error, "error: {}", result.output);
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "replaced\nbar\nreplaced\nbaz\n");
    }

    #[tokio::test]
    async fn patch_path_traversal_blocked() {
        let (tool, _dir) = setup();
        let result = call_tool(
            &tool,
            serde_json::json!({
                "path": "../../etc/passwd",
                "old": "x",
                "new": "y",
                "strategy": "exact"
            }),
        )
        .await;
        assert!(result.is_error);
        assert!(result.output.contains("path traversal"));
    }

    #[tokio::test]
    async fn patch_file_not_found() {
        let (tool, _dir) = setup();
        let result = call_tool(
            &tool,
            serde_json::json!({
                "path": "nonexistent.txt",
                "old": "x",
                "new": "y",
                "strategy": "exact"
            }),
        )
        .await;
        assert!(result.is_error);
        assert!(result.output.contains("failed to read file"));
    }
}
