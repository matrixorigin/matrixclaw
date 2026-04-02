use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

fn validate_path(workspace_root: &str, path: &str) -> Result<PathBuf, String> {
    let workspace = Path::new(workspace_root)
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

pub struct ReadFileTool {
    descriptor: ToolDescriptor,
    workspace_root: String,
}

impl ReadFileTool {
    pub fn new(workspace_root: &str) -> Self {
        Self {
            descriptor: ToolDescriptor::new("read_file", "Read file contents").with_parameters(
                vec![
                    ToolParameter::required(
                        "path",
                        ParameterType::String,
                        "Path to the file (relative to workspace)",
                    ),
                    ToolParameter::optional(
                        "offset",
                        ParameterType::Integer,
                        "Starting line number (1-indexed)",
                    ),
                    ToolParameter::optional(
                        "limit",
                        ParameterType::Integer,
                        "Maximum number of lines to return",
                    ),
                ],
            ),
            workspace_root: workspace_root.to_string(),
        }
    }
}

#[async_trait]
impl ToolExecutor for ReadFileTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let path = match call.arguments.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult::error(&call, "missing required parameter: path"),
        };

        let validated = match validate_path(&self.workspace_root, path) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(&call, e),
        };

        let content = match tokio::fs::read_to_string(&validated).await {
            Ok(c) => c,
            Err(e) => return ToolResult::error(&call, format!("failed to read file: {e}")),
        };

        let lines: Vec<&str> = content.lines().collect();
        let offset = call
            .arguments
            .get("offset")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;
        let limit = call
            .arguments
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|l| l as usize);

        let start = offset.saturating_sub(1);
        let end = match limit {
            Some(l) => (start + l).min(lines.len()),
            None => lines.len(),
        };

        let result: Vec<String> = lines[start..end]
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{}: {}", start + i + 1, line))
            .collect();

        ToolResult::success(&call, result.join("\n"))
    }
}

pub struct WriteFileTool {
    descriptor: ToolDescriptor,
    workspace_root: String,
}

impl WriteFileTool {
    pub fn new(workspace_root: &str) -> Self {
        Self {
            descriptor: ToolDescriptor::new("write_file", "Write content to a file")
                .with_parameters(vec![
                    ToolParameter::required(
                        "path",
                        ParameterType::String,
                        "Path to the file (relative to workspace)",
                    ),
                    ToolParameter::required("content", ParameterType::String, "Content to write"),
                ]),
            workspace_root: workspace_root.to_string(),
        }
    }
}

#[async_trait]
impl ToolExecutor for WriteFileTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let path = match call.arguments.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult::error(&call, "missing required parameter: path"),
        };

        let content = match call.arguments.get("content").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return ToolResult::error(&call, "missing required parameter: content"),
        };

        let validated = match validate_path(&self.workspace_root, path) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(&call, e),
        };

        if let Some(parent) = validated.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                return ToolResult::error(&call, format!("failed to create directories: {e}"));
            }
        }

        match tokio::fs::write(&validated, content).await {
            Ok(_) => ToolResult::success(&call, format!("wrote {path}")),
            Err(e) => ToolResult::error(&call, format!("failed to write file: {e}")),
        }
    }
}

pub struct ListDirectoryTool {
    descriptor: ToolDescriptor,
    workspace_root: String,
}

impl ListDirectoryTool {
    pub fn new(workspace_root: &str) -> Self {
        Self {
            descriptor: ToolDescriptor::new("list_directory", "List directory contents")
                .with_parameters(vec![
                    ToolParameter::required(
                        "path",
                        ParameterType::String,
                        "Path to the directory (relative to workspace)",
                    ),
                    ToolParameter::optional(
                        "recursive",
                        ParameterType::Boolean,
                        "List recursively (default false)",
                    ),
                ]),
            workspace_root: workspace_root.to_string(),
        }
    }
}

#[async_trait]
impl ToolExecutor for ListDirectoryTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let path = match call.arguments.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult::error(&call, "missing required parameter: path"),
        };

        let validated = match validate_path(&self.workspace_root, path) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(&call, e),
        };

        let recursive = call
            .arguments
            .get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut entries = Vec::new();
        if let Err(e) = list_entries(&validated, &validated, recursive, &mut entries).await {
            return ToolResult::error(&call, format!("failed to list directory: {e}"));
        }

        ToolResult::success(&call, entries.join("\n"))
    }
}

async fn list_entries(
    base: &Path,
    current: &Path,
    recursive: bool,
    entries: &mut Vec<String>,
) -> std::io::Result<()> {
    let mut reader = tokio::fs::read_dir(current).await?;
    let mut subdirs = Vec::new();

    while let Some(entry) = reader.next_entry().await? {
        let name = entry.file_name().to_string_lossy().to_string();
        let file_type = entry.file_type().await?;
        let relative = current.strip_prefix(base).unwrap_or(current).to_path_buf();

        let prefix = if relative == PathBuf::new() {
            String::new()
        } else {
            format!("{}/", relative.display())
        };

        if file_type.is_dir() {
            entries.push(format!("{prefix}{name}/"));
            if recursive {
                subdirs.push(entry.path());
            }
        } else {
            entries.push(format!("{prefix}{name}"));
        }
    }

    entries.sort();

    if recursive {
        for subdir in subdirs {
            Box::pin(list_entries(base, &subdir, true, entries)).await?;
        }
    }

    Ok(())
}

pub struct EditFileTool {
    descriptor: ToolDescriptor,
    workspace_root: String,
}

impl EditFileTool {
    pub fn new(workspace_root: &str) -> Self {
        Self {
            descriptor: ToolDescriptor::new("edit_file", "Edit a file by replacing text")
                .with_parameters(vec![
                    ToolParameter::required(
                        "path",
                        ParameterType::String,
                        "Path to the file (relative to workspace)",
                    ),
                    ToolParameter::required(
                        "old_text",
                        ParameterType::String,
                        "Text to find in the file",
                    ),
                    ToolParameter::required(
                        "new_text",
                        ParameterType::String,
                        "Text to replace with",
                    ),
                ]),
            workspace_root: workspace_root.to_string(),
        }
    }
}

#[async_trait]
impl ToolExecutor for EditFileTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let path = match call.arguments.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult::error(&call, "missing required parameter: path"),
        };

        let old_text = match call.arguments.get("old_text").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => return ToolResult::error(&call, "missing required parameter: old_text"),
        };

        let new_text = match call.arguments.get("new_text").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => return ToolResult::error(&call, "missing required parameter: new_text"),
        };

        let validated = match validate_path(&self.workspace_root, path) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(&call, e),
        };

        let content = match tokio::fs::read_to_string(&validated).await {
            Ok(c) => c,
            Err(e) => return ToolResult::error(&call, format!("failed to read file: {e}")),
        };

        if !content.contains(old_text) {
            return ToolResult::error(&call, "old_text not found in file");
        }

        let count = content.matches(old_text).count();
        let new_content = content.replace(old_text, new_text);

        match tokio::fs::write(&validated, new_content).await {
            Ok(_) => {
                ToolResult::success(&call, format!("replaced {count} occurrence(s) in {path}"))
            }
            Err(e) => ToolResult::error(&call, format!("failed to write file: {e}")),
        }
    }
}
