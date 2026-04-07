use std::fs;
use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct SkillsTool {
    descriptor: ToolDescriptor,
    skills_dir: PathBuf,
}

impl SkillsTool {
    pub fn new(home: &Path) -> Self {
        let skills_dir = home.join(".matrixclaw").join("skills");
        Self {
            descriptor: ToolDescriptor::new(
                "skills",
                "List, read, and create reusable skills. Skills are Markdown files stored in ~/.matrixclaw/skills/.",
            )
            .with_parameters(vec![
                ToolParameter::required("action", ParameterType::String, "Action to perform")
                    .enum_values(&["list", "read", "create", "history"]),
                ToolParameter::optional("name", ParameterType::String, "Skill name (for read/create)"),
                ToolParameter::optional("content", ParameterType::String, "Skill content in Markdown (for create)"),
            ]),
            skills_dir,
        }
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
impl ToolExecutor for SkillsTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let action = match call.arguments.get("action").and_then(|v| v.as_str()) {
            Some(a) => a,
            None => return ToolResult::error(&call, "missing required parameter: action"),
        };

        match action {
            "list" => self.handle_list(&call).await,
            "read" => self.handle_read(&call).await,
            "create" => self.handle_create(&call).await,
            "history" => self.handle_history(&call),
            _ => ToolResult::error(&call, format!("unknown action: {action}")),
        }
    }
}

impl SkillsTool {
    async fn handle_list(&self, call: &ToolCall) -> ToolResult {
        if !self.skills_dir.exists() {
            return ToolResult::success(call, "(no skills installed)");
        }
        let entries = match fs::read_dir(&self.skills_dir) {
            Ok(e) => e,
            Err(e) => return ToolResult::error(call, format!("failed to list skills: {e}")),
        };
        let mut names: Vec<String> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
            .filter(|name| self.skill_entry(name).exists())
            .collect();
        names.sort();
        if names.is_empty() {
            ToolResult::success(call, "(no skills installed)")
        } else {
            ToolResult::success(call, names.join("\n"))
        }
    }

    async fn handle_read(&self, call: &ToolCall) -> ToolResult {
        let name = match call.arguments.get("name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => return ToolResult::error(call, "missing required parameter: name"),
        };
        let entry_path = self.skill_entry(name);
        if !entry_path.exists() {
            return ToolResult::error(call, format!("skill not found: {name}"));
        }
        match fs::read_to_string(&entry_path) {
            Ok(content) => ToolResult::success(call, content),
            Err(e) => ToolResult::error(call, format!("failed to read skill: {e}")),
        }
    }

    async fn handle_create(&self, call: &ToolCall) -> ToolResult {
        let name = match call.arguments.get("name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => return ToolResult::error(call, "missing required parameter: name"),
        };
        let content = match call.arguments.get("content").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return ToolResult::error(call, "missing required parameter: content"),
        };
        if name.contains('/') || name.contains('\\') || name.contains('.') {
            return ToolResult::error(call, "skill name must not contain /, \\, or .");
        }
        if name.is_empty() {
            return ToolResult::error(call, "skill name must not be empty");
        }
        let skill_dir = self.skills_dir.join(name);
        if let Err(e) = fs::create_dir_all(&skill_dir) {
            return ToolResult::error(call, format!("failed to create skill directory: {e}"));
        }
        if let Err(e) = self.archive_current(name) {
            return ToolResult::error(call, e);
        }
        let entry_path = skill_dir.join("SKILL.md");
        if let Err(e) = fs::write(&entry_path, content) {
            return ToolResult::error(call, format!("failed to write skill: {e}"));
        }
        ToolResult::success(call, format!("created skill: {name}"))
    }

    fn archive_current(&self, name: &str) -> Result<(), String> {
        let skill_dir = self.skills_dir.join(name);
        let skill_md = skill_dir.join("SKILL.md");
        if !skill_md.exists() {
            return Ok(());
        }
        let existing: Vec<_> = fs::read_dir(&skill_dir)
            .map_err(|e| format!("failed to read skill directory: {e}"))?
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
            .filter(|n| n.starts_with('v') && n.ends_with(".md"))
            .collect();
        let version_num = existing.len() + 1;
        let archive_name = format!("v{version_num}.md");
        fs::copy(&skill_md, skill_dir.join(&archive_name))
            .map_err(|e| format!("failed to archive skill: {e}"))?;
        Ok(())
    }

    fn handle_history(&self, call: &ToolCall) -> ToolResult {
        let name = match call.arguments.get("name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => return ToolResult::error(call, "missing required parameter: name"),
        };
        let skill_dir = self.skills_dir.join(name);
        if !skill_dir.exists() {
            return ToolResult::error(call, format!("skill not found: {name}"));
        }
        let mut versions: Vec<String> = Vec::new();
        let skill_md = skill_dir.join("SKILL.md");
        if skill_md.exists() {
            versions.push("current (SKILL.md)".to_string());
        }
        let mut archived: Vec<String> = fs::read_dir(&skill_dir)
            .unwrap_or_else(|_| panic!("failed to read dir"))
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
            .filter(|n| n.starts_with('v') && n.ends_with(".md"))
            .collect();
        archived.sort();
        versions.extend(archived);
        if versions.is_empty() {
            ToolResult::success(call, "(no versions)")
        } else {
            ToolResult::success(call, versions.join("\n"))
        }
    }

    fn skill_entry(&self, name: &str) -> PathBuf {
        self.skills_dir.join(name).join("SKILL.md")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_tool() -> (SkillsTool, TempDir) {
        let dir = TempDir::new().unwrap();
        let tool = SkillsTool::new(dir.path());
        (tool, dir)
    }

    async fn call(tool: &SkillsTool, args: &str) -> ToolResult {
        let call = ToolCall::new(
            "1".into(),
            "skills".into(),
            serde_json::json!(serde_json::from_str::<serde_json::Value>(args).unwrap()),
        );
        tool.execute(call).await
    }

    #[tokio::test]
    async fn list_empty() {
        let (tool, _dir) = make_tool();
        let r = call(&tool, r#"{"action":"list"}"#).await;
        assert!(!r.is_error);
        assert!(r.output.contains("no skills"));
    }

    #[tokio::test]
    async fn create_and_read() {
        let (tool, _dir) = make_tool();
        let r = call(
            &tool,
            r#"{"action":"create","name":"test-skill","content":"Hello world"}"#,
        )
        .await;
        assert!(!r.is_error);
        let r = call(&tool, r#"{"action":"read","name":"test-skill"}"#).await;
        assert!(!r.is_error);
        assert!(r.output.contains("Hello world"));
    }

    #[tokio::test]
    async fn list_shows_created_skill() {
        let (tool, _dir) = make_tool();
        call(
            &tool,
            r#"{"action":"create","name":"my-skill","content":"content"}"#,
        )
        .await;
        let r = call(&tool, r#"{"action":"list"}"#).await;
        assert!(!r.is_error);
        assert!(r.output.contains("my-skill"));
    }

    #[tokio::test]
    async fn read_missing_skill() {
        let (tool, _dir) = make_tool();
        let r = call(&tool, r#"{"action":"read","name":"nope"}"#).await;
        assert!(r.is_error);
        assert!(r.output.contains("not found"));
    }

    #[tokio::test]
    async fn create_rejects_path_traversal() {
        let (tool, _dir) = make_tool();
        let r = call(
            &tool,
            r#"{"action":"create","name":"../evil","content":"x"}"#,
        )
        .await;
        assert!(r.is_error);
        assert!(r.output.contains("must not contain"));
    }

    #[tokio::test]
    async fn create_overwrites_existing() {
        let (tool, _dir) = make_tool();
        call(&tool, r#"{"action":"create","name":"s","content":"v1"}"#).await;
        call(&tool, r#"{"action":"create","name":"s","content":"v2"}"#).await;
        let r = call(&tool, r#"{"action":"read","name":"s"}"#).await;
        assert_eq!(r.output, "v2");
    }

    #[tokio::test]
    async fn create_archives_previous_version() {
        let (tool, _dir) = make_tool();
        call(&tool, r#"{"action":"create","name":"v1","content":"v1"}"#).await;
        call(&tool, r#"{"action":"create","name":"v1","content":"v2"}"#).await;
        let r = call(&tool, r#"{"action":"history","name":"v1"}"#).await;
        assert!(!r.is_error);
        assert!(r.output.contains("current (SKILL.md)"));
        assert!(r.output.contains("v1.md"));
        let r = call(&tool, r#"{"action":"read","name":"v1"}"#).await;
        assert_eq!(r.output, "v2");
    }

    #[tokio::test]
    async fn history_shows_no_versions_for_new_skill() {
        let (tool, _dir) = make_tool();
        call(&tool, r#"{"action":"create","name":"fresh","content":"content"}"#).await;
        let r = call(&tool, r#"{"action":"history","name":"fresh"}"#).await;
        assert!(!r.is_error);
        assert!(r.output.contains("current (SKILL.md)"));
        assert!(!r.output.contains("v1.md"));
    }

    #[tokio::test]
    async fn history_multiple_versions() {
        let (tool, _dir) = make_tool();
        call(&tool, r#"{"action":"create","name":"multi","content":"first"}"#).await;
        call(&tool, r#"{"action":"create","name":"multi","content":"second"}"#).await;
        call(&tool, r#"{"action":"create","name":"multi","content":"third"}"#).await;
        let r = call(&tool, r#"{"action":"history","name":"multi"}"#).await;
        assert!(!r.is_error);
        assert!(r.output.contains("v1.md"));
        assert!(r.output.contains("v2.md"));
        assert!(r.output.contains("current (SKILL.md)"));
        let r = call(&tool, r#"{"action":"read","name":"multi"}"#).await;
        assert_eq!(r.output, "third");
    }
}
