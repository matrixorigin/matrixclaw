use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::compat_registry::CompatRegistryEntry;
use crate::paths;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledSkillRecord {
    pub name: String,
    pub source_root: PathBuf,
    pub installed_root: PathBuf,
    pub manifest_path: PathBuf,
    pub provenance_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnabledSkillsRecord {
    pub agent_name: String,
    pub enabled: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillsInventory {
    pub installed: Vec<InstalledSkillRecord>,
    pub enabled: Vec<EnabledSkillsRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnableSkillChange {
    pub agent_name: String,
    pub skill_name: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct EnabledSkillsFile {
    #[serde(rename = "schemaVersion")]
    schema_version: String,
    #[serde(rename = "agentName")]
    agent_name: String,
    enabled: Vec<String>,
}

pub fn compat_registry_path(home: impl AsRef<Path>) -> PathBuf {
    paths::runtime_home(home)
        .join("state")
        .join("compat-registry.json")
}

pub fn enabled_skills_path(home: impl AsRef<Path>, agent_name: impl AsRef<str>) -> PathBuf {
    paths::runtime_home(home)
        .join("agents")
        .join(agent_name.as_ref())
        .join("enabled-skills.json")
}

pub fn skills_inventory_for_agent(
    home: impl AsRef<Path>,
    agent_name: impl AsRef<str>,
) -> io::Result<SkillsInventory> {
    let installed = load_installed_skills(home.as_ref())?;
    let enabled = load_enabled_skills(home.as_ref(), agent_name.as_ref())?;

    Ok(SkillsInventory { installed, enabled })
}

pub fn set_skill_enabled(
    home: impl AsRef<Path>,
    change: &EnableSkillChange,
) -> io::Result<EnabledSkillsRecord> {
    let mut current = load_enabled_skills(home.as_ref(), &change.agent_name)?
        .into_iter()
        .next()
        .unwrap_or(EnabledSkillsRecord {
            agent_name: change.agent_name.clone(),
            enabled: Vec::new(),
        });

    if change.enabled {
        if !current.enabled.contains(&change.skill_name) {
            current.enabled.push(change.skill_name.clone());
        }
    } else {
        current.enabled.retain(|name| name != &change.skill_name);
    }

    current.enabled.sort();
    current.enabled.dedup();

    save_enabled_skills(home.as_ref(), &current)?;
    Ok(current)
}

fn load_installed_skills(home: impl AsRef<Path>) -> io::Result<Vec<InstalledSkillRecord>> {
    let registry_path = compat_registry_path(home);
    if !registry_path.exists() {
        return Ok(Vec::new());
    }

    let body = fs::read_to_string(&registry_path)?;
    let entries = load_registry_entries(&body)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    Ok(entries
        .into_iter()
        .filter(|entry| entry.kind == "skill")
        .map(|entry| InstalledSkillRecord {
            name: entry.name,
            source_root: entry.source_root,
            installed_root: entry.installed_root,
            manifest_path: entry.manifest_path,
            provenance_path: entry.provenance_path,
        })
        .collect())
}

fn load_enabled_skills(
    home: impl AsRef<Path>,
    agent_name: impl AsRef<str>,
) -> io::Result<Vec<EnabledSkillsRecord>> {
    let path = enabled_skills_path(home, agent_name.as_ref());
    if !path.exists() {
        return Ok(Vec::new());
    }

    let body = fs::read_to_string(path)?;
    let parsed: EnabledSkillsFile = serde_json::from_str(&body)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    Ok(vec![EnabledSkillsRecord {
        agent_name: parsed.agent_name,
        enabled: parsed.enabled,
    }])
}

fn save_enabled_skills(
    home: impl AsRef<Path>,
    record: &EnabledSkillsRecord,
) -> io::Result<PathBuf> {
    let path = enabled_skills_path(home, &record.agent_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let body = serde_json::to_string_pretty(&EnabledSkillsFile {
        schema_version: "1".to_string(),
        agent_name: record.agent_name.clone(),
        enabled: record.enabled.clone(),
    })
    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    fs::write(&path, body)?;
    Ok(path)
}

fn load_registry_entries(body: &str) -> serde_json::Result<Vec<CompatRegistryEntry>> {
    match serde_json::from_str::<Vec<CompatRegistryEntry>>(body) {
        Ok(entries) => Ok(entries),
        Err(list_error) => match serde_json::from_str::<CompatRegistryEntry>(body) {
            Ok(entry) => Ok(vec![entry]),
            Err(single_error) => {
                if single_error.is_data() || single_error.is_syntax() {
                    Err(list_error)
                } else {
                    Err(single_error)
                }
            }
        },
    }
}
