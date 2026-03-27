use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportProvenance {
    pub schema_version: String,
    pub origin: String,
    pub source_root: PathBuf,
    pub installed_root: PathBuf,
    pub manifest_path: PathBuf,
    pub original_files: Vec<String>,
    pub generated_files: Vec<String>,
    pub artifact_class: Option<String>,
    pub support_tier: Option<String>,
    pub installed_at: Option<String>,
}

impl ImportProvenance {
    pub fn new(
        origin: impl Into<String>,
        source_root: impl Into<PathBuf>,
        installed_root: impl Into<PathBuf>,
        manifest_path: impl Into<PathBuf>,
        original_files: Vec<String>,
    ) -> Self {
        Self {
            schema_version: "1".to_string(),
            origin: origin.into(),
            source_root: source_root.into(),
            installed_root: installed_root.into(),
            manifest_path: manifest_path.into(),
            original_files,
            generated_files: Vec::new(),
            artifact_class: None,
            support_tier: None,
            installed_at: None,
        }
    }

    pub fn with_generated_files(mut self, generated_files: Vec<String>) -> Self {
        self.generated_files = generated_files;
        self
    }

    pub fn with_compatibility(
        mut self,
        artifact_class: impl Into<String>,
        support_tier: impl Into<String>,
    ) -> Self {
        self.artifact_class = Some(artifact_class.into());
        self.support_tier = Some(support_tier.into());
        self
    }

    pub fn with_installed_at_now(mut self) -> Self {
        self.installed_at = Some(current_install_timestamp());
        self
    }

    pub fn save_to(&self, path: impl AsRef<Path>) -> io::Result<PathBuf> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialized = serde_json::to_string_pretty(self)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        fs::write(path, serialized)?;
        Ok(path.to_path_buf())
    }
}

fn current_install_timestamp() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => format!("unix:{}.{:09}", duration.as_secs(), duration.subsec_nanos()),
        Err(_) => "unix:0.000000000".to_string(),
    }
}
