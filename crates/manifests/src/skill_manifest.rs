use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::provenance::ImportProvenance;

pub const SKILL_ENTRY_NAME: &str = "SKILL.md";
pub const NORMALIZED_MANIFEST_NAME: &str = "matrixclaw.skill.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupportTier {
    Native,
    Shimmed,
    BridgeOnly,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillDetection {
    pub name: String,
    pub root: PathBuf,
    pub entry: PathBuf,
    pub description: String,
    pub origin: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillInstallOutcome {
    Imported {
        manifest_path: PathBuf,
        installed_root: PathBuf,
        provenance_path: PathBuf,
    },
    Rejected {
        reason: String,
    },
}

pub fn detect_skill_root(root: impl AsRef<Path>) -> io::Result<Option<SkillDetection>> {
    let root = root.as_ref();
    let entry = root.join(SKILL_ENTRY_NAME);
    if !entry.exists() {
        return Ok(None);
    }

    let name = root
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("skill")
        .to_string();
    let description =
        read_skill_description(&entry).unwrap_or_else(|_| format!("Imported skill package {name}"));

    Ok(Some(SkillDetection {
        name,
        root: root.to_path_buf(),
        entry,
        description,
        origin: "openclaw".to_string(),
    }))
}

pub fn normalize_skill_manifest(detection: &SkillDetection) -> serde_json::Value {
    serde_json::json!({
        "schemaVersion": "1",
        "name": detection.name,
        "version": "1.0.0",
        "description": detection.description,
        "entry": SKILL_ENTRY_NAME,
        "compat": {
            "origin": detection.origin,
            "artifactClass": "skill_text",
            "tier": "native",
            "importMode": "normalized"
        },
        "source": {
            "type": "local_path",
            "ref": detection.root,
            "path": detection.root,
        },
        "provenance": {
            "originalFiles": [SKILL_ENTRY_NAME]
        }
    })
}

pub fn import_skill_package(
    source_root: impl AsRef<Path>,
    runtime_home: impl AsRef<Path>,
) -> io::Result<SkillInstallOutcome> {
    let detection = match detect_skill_root(&source_root)? {
        Some(detection) => detection,
        None => {
            return Ok(SkillInstallOutcome::Rejected {
                reason: "missing SKILL.md".to_string(),
            })
        }
    };

    let manifest = normalize_skill_manifest(&detection);
    let installed_root = runtime_home
        .as_ref()
        .join(".matrixclaw")
        .join("skills")
        .join(&detection.name);
    let manifest_path = installed_root.join(NORMALIZED_MANIFEST_NAME);
    let provenance_path = installed_root.join("provenance.json");

    fs::create_dir_all(&installed_root)?;
    fs::write(
        installed_root.join(SKILL_ENTRY_NAME),
        fs::read_to_string(&detection.entry)?,
    )?;
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
    )?;

    let provenance = ImportProvenance::new(
        detection.origin.clone(),
        detection.root.clone(),
        installed_root.clone(),
        manifest_path.clone(),
        vec![SKILL_ENTRY_NAME.to_string()],
    );
    provenance.save_to(&provenance_path)?;

    Ok(SkillInstallOutcome::Imported {
        manifest_path,
        installed_root,
        provenance_path,
    })
}

fn read_skill_description(path: &Path) -> io::Result<String> {
    let content = fs::read_to_string(path)?;
    let description = content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .unwrap_or("Imported skill package");
    Ok(description.to_string())
}
