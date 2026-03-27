use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::paths;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserAssetManifest {
    pub schema_version: String,
    pub name: String,
    pub version: String,
    pub cached_path: PathBuf,
}

impl BrowserAssetManifest {
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        cached_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            schema_version: "1".to_string(),
            name: name.into(),
            version: version.into(),
            cached_path: cached_path.into(),
        }
    }

    pub fn manifest_path(home: impl AsRef<Path>) -> PathBuf {
        paths::managed_assets_dir(home)
            .join("browser")
            .join("browser-asset.manifest")
    }

    pub fn save_to_home(&self, home: impl AsRef<Path>) -> io::Result<PathBuf> {
        let path = Self::manifest_path(home);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let body = format!(
            "schemaVersion={}\nname={}\nversion={}\ncachedPath={}\n",
            self.schema_version,
            self.name,
            self.version,
            self.cached_path.display()
        );
        fs::write(&path, body)?;
        Ok(path)
    }

    pub fn load_from_home(home: impl AsRef<Path>) -> io::Result<Option<Self>> {
        let path = Self::manifest_path(home);
        if !path.exists() {
            return Ok(None);
        }

        let body = fs::read_to_string(&path)?;
        let mut schema_version = String::new();
        let mut name = String::new();
        let mut version = String::new();
        let mut cached_path = PathBuf::new();

        for line in body.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key {
                "schemaVersion" => schema_version = value.to_string(),
                "name" => name = value.to_string(),
                "version" => version = value.to_string(),
                "cachedPath" => cached_path = PathBuf::from(value),
                _ => {}
            }
        }

        Ok(Some(Self {
            schema_version,
            name,
            version,
            cached_path,
        }))
    }
}
