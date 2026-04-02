use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use matrixclaw_manifests::plugin_manifest::PluginInstallOutcome;
use matrixclaw_manifests::skill_manifest::SkillInstallOutcome;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatRegistryEntry {
    pub schema_version: String,
    pub kind: String,
    pub name: String,
    pub source_root: PathBuf,
    pub installed_root: PathBuf,
    pub manifest_path: PathBuf,
    pub provenance_path: PathBuf,
    pub support_tier: Option<String>,
    pub adapter_path: Option<PathBuf>,
    pub capabilities: Vec<String>,
}

pub struct PluginInstallPaths {
    pub source_root: PathBuf,
    pub installed_root: PathBuf,
    pub manifest_path: PathBuf,
    pub provenance_path: PathBuf,
    pub adapter_path: PathBuf,
}

pub struct PluginInstallMeta {
    pub support_tier: String,
    pub capabilities: Vec<String>,
}

impl CompatRegistryEntry {
    pub fn from_skill_install(
        name: impl Into<String>,
        source_root: impl Into<PathBuf>,
        installed_root: impl Into<PathBuf>,
        manifest_path: impl Into<PathBuf>,
        provenance_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            schema_version: "1".to_string(),
            kind: "skill".to_string(),
            name: name.into(),
            source_root: source_root.into(),
            installed_root: installed_root.into(),
            manifest_path: manifest_path.into(),
            provenance_path: provenance_path.into(),
            support_tier: None,
            adapter_path: None,
            capabilities: Vec::new(),
        }
    }

    pub fn from_plugin_install(
        name: impl Into<String>,
        paths: PluginInstallPaths,
        meta: PluginInstallMeta,
    ) -> Self {
        Self {
            schema_version: "1".to_string(),
            kind: "plugin".to_string(),
            name: name.into(),
            source_root: paths.source_root,
            installed_root: paths.installed_root,
            manifest_path: paths.manifest_path,
            provenance_path: paths.provenance_path,
            support_tier: Some(meta.support_tier),
            adapter_path: Some(paths.adapter_path),
            capabilities: meta.capabilities,
        }
    }

    pub fn save_to(&self, path: impl AsRef<Path>) -> io::Result<PathBuf> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let body = serde_json::to_string_pretty(self)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        fs::write(path, body)?;
        Ok(path.to_path_buf())
    }
}

pub fn record_skill_install(
    runtime_home: impl AsRef<Path>,
    source_root: impl AsRef<Path>,
    outcome: &SkillInstallOutcome,
) -> io::Result<Option<PathBuf>> {
    let SkillInstallOutcome::Imported {
        manifest_path,
        installed_root,
        provenance_path,
    } = outcome
    else {
        return Ok(None);
    };

    let entry = CompatRegistryEntry::from_skill_install(
        installed_root
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("skill")
            .to_string(),
        source_root.as_ref().to_path_buf(),
        installed_root.clone(),
        manifest_path.clone(),
        provenance_path.clone(),
    );
    let registry_path = runtime_home
        .as_ref()
        .join(".matrixclaw")
        .join("state")
        .join("compat-registry.json");
    let _ = entry.save_to(&registry_path)?;
    Ok(Some(registry_path))
}

pub fn record_plugin_install(
    runtime_home: impl AsRef<Path>,
    source_root: impl AsRef<Path>,
    outcome: &PluginInstallOutcome,
) -> io::Result<Option<PathBuf>> {
    let PluginInstallOutcome::Imported {
        manifest_path,
        installed_root,
        provenance_path,
        adapter_path,
    } = outcome
    else {
        return Ok(None);
    };

    let (support_tier, capabilities) = read_plugin_registry_metadata(adapter_path)?;
    let entry = CompatRegistryEntry::from_plugin_install(
        installed_root
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("plugin")
            .to_string(),
        PluginInstallPaths {
            source_root: source_root.as_ref().to_path_buf(),
            installed_root: installed_root.clone(),
            manifest_path: manifest_path.clone(),
            provenance_path: provenance_path.clone(),
            adapter_path: adapter_path.clone(),
        },
        PluginInstallMeta {
            support_tier,
            capabilities,
        },
    );
    let registry_path = runtime_home
        .as_ref()
        .join(".matrixclaw")
        .join("state")
        .join("compat-registry.json");
    let _ = entry.save_to(&registry_path)?;
    Ok(Some(registry_path))
}

fn read_plugin_registry_metadata(path: &Path) -> io::Result<(String, Vec<String>)> {
    #[derive(Deserialize)]
    struct AdapterContract {
        support_tier: String,
        capabilities: Vec<String>,
    }

    let content = fs::read_to_string(path)?;
    let contract: AdapterContract = serde_json::from_str(&content)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok((contract.support_tier, contract.capabilities))
}
