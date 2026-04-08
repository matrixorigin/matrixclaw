use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct SearchFilesTool {
    descriptor: ToolDescriptor,
    workspace_root: String,
}

impl SearchFilesTool {
    pub fn new(workspace_root: &str) -> Self {
        Self {
            descriptor: ToolDescriptor::new(
                "search_files",
                "Search file contents using regex patterns. Returns matching file paths with line numbers and content.",
            )
            .with_parameters(vec![
                ToolParameter::required("pattern", ParameterType::String, "Regex pattern to search for"),
                ToolParameter::optional("path", ParameterType::String, "Subdirectory to search in (relative to workspace)"),
                ToolParameter::optional("include", ParameterType::String, "File glob filter (e.g. *.rs, *.py)"),
                ToolParameter::optional("max_results", ParameterType::String, "Maximum number of results (default 50)"),
            ]),
            workspace_root: workspace_root.to_string(),
        }
    }
}

fn resolve_search_path(workspace_root: &str, subpath: Option<&str>) -> Result<PathBuf, String> {
    let workspace = Path::new(workspace_root)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(workspace_root));

    if let Some(p) = subpath {
        if p.contains("..") {
            return Err("path traversal detected: path is outside workspace".to_string());
        }
    }

    let target = match subpath {
        Some(p) => workspace.join(p),
        None => workspace.clone(),
    };

    match target.canonicalize() {
        Ok(canonical) => {
            if canonical.starts_with(&workspace) {
                Ok(canonical)
            } else {
                Err("path traversal detected: path is outside workspace".to_string())
            }
        }
        Err(_) => Ok(target),
    }
}

#[async_trait]
impl ToolExecutor for SearchFilesTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let pattern = match call.arguments.get("pattern").and_then(|v| v.as_str()) {
            Some(p) => p.to_string(),
            None => return ToolResult::error(&call, "missing required parameter: pattern"),
        };

        let subpath = call.arguments.get("path").and_then(|v| v.as_str());

        let search_dir = match resolve_search_path(&self.workspace_root, subpath) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(&call, e),
        };

        let max_results = call
            .arguments
            .get("max_results")
            .and_then(|v| v.as_str())
            .unwrap_or("50");

        let mut cmd = std::process::Command::new("rg");
        cmd.arg("--no-heading")
            .arg("--line-number")
            .arg("--max-count")
            .arg(max_results)
            .arg(&pattern);

        if let Some(include) = call.arguments.get("include").and_then(|v| v.as_str()) {
            cmd.arg("--glob").arg(include);
        }

        cmd.arg(&search_dir);

        let result = cmd.output();

        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                match output.status.code() {
                    Some(0) => ToolResult::success(&call, stdout),
                    Some(1) => ToolResult::success(&call, "no matches found"),
                    Some(code) => {
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        ToolResult::error(&call, format!("rg exited with code {code}: {stderr}"))
                    }
                    None => ToolResult::error(&call, "rg was terminated by signal"),
                }
            }
            Err(e) => ToolResult::error(&call, format!("failed to execute rg: {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> (SearchFilesTool, TempDir) {
        let dir = TempDir::new().unwrap();
        let tool = SearchFilesTool::new(dir.path().to_str().unwrap());
        (tool, dir)
    }

    async fn call(tool: &SearchFilesTool, args: &str) -> ToolResult {
        let call = ToolCall::new(
            "1".into(),
            "search_files".into(),
            serde_json::json!(serde_json::from_str::<serde_json::Value>(args).unwrap()),
        );
        tool.execute(call).await
    }

    #[tokio::test]
    async fn finds_matching_content() {
        let (tool, dir) = setup();
        fs::write(dir.path().join("hello.txt"), "hello world\nfoo bar\n").unwrap();
        let result = call(&tool, r#"{"pattern": "hello"}"#).await;
        assert!(!result.is_error);
        assert!(result.output.contains("hello"));
    }

    #[tokio::test]
    async fn no_matches_returns_empty() {
        let (tool, dir) = setup();
        fs::write(dir.path().join("hello.txt"), "hello world\n").unwrap();
        let result = call(&tool, r#"{"pattern": "zzzzz_not_found"}"#).await;
        assert!(!result.is_error);
        assert!(result.output.contains("no matches"));
    }

    #[tokio::test]
    async fn respects_include_glob() {
        let (tool, dir) = setup();
        fs::write(
            dir.path().join("code.rs"),
            "fn main() { println!(\"rust_match\"); }\n",
        )
        .unwrap();
        fs::write(dir.path().join("notes.txt"), "rust_match in notes\n").unwrap();
        let result = call(&tool, r#"{"pattern": "rust_match", "include": "*.rs"}"#).await;
        assert!(!result.is_error);
        assert!(result.output.contains("code.rs"));
        assert!(!result.output.contains("notes.txt"));
    }

    #[tokio::test]
    async fn limits_results() {
        let (tool, dir) = setup();
        let content: String = (0..10).map(|i| format!("match_line_{i}\n")).collect();
        fs::write(dir.path().join("many.txt"), content).unwrap();
        let result = call(&tool, r#"{"pattern": "match_line_", "max_results": "2"}"#).await;
        assert!(!result.is_error);
        let line_count = result.output.lines().count();
        assert!(line_count <= 2);
    }

    #[tokio::test]
    async fn restricts_to_workspace() {
        let (tool, _dir) = setup();
        let result = call(
            &tool,
            r#"{"pattern": "anything", "path": "../../etc/passwd"}"#,
        )
        .await;
        assert!(result.is_error);
        assert!(result.output.contains("path traversal"));
    }
}
