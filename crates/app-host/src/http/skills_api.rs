use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::compat_registry::CompatRegistryEntry;
use crate::http::{HttpRequest, HttpResponse, SetupSurface};
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
pub struct SkillCatalogRecord {
    pub name: String,
    pub source_root: PathBuf,
    pub installed_root: PathBuf,
    pub enabled_by_agent_count: usize,
    pub enabled_by_agents: Vec<String>,
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

pub const SKILLS_INVENTORY_ROUTE: &str = "/api/skills";
pub const SKILLS_TOGGLE_ROUTE: &str = "/api/skills/toggle";
pub const SKILLS_CATALOG_ROUTE: &str = "/api/skills/catalog";

pub fn is_skills_inventory_route(path: &str) -> bool {
    crate::http::routes::normalize_path(path) == SKILLS_INVENTORY_ROUTE
}

pub fn is_skills_toggle_route(path: &str) -> bool {
    crate::http::routes::normalize_path(path) == SKILLS_TOGGLE_ROUTE
}

pub fn is_skills_catalog_route(path: &str) -> bool {
    crate::http::routes::normalize_path(path) == SKILLS_CATALOG_ROUTE
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

pub fn skills_catalog_for_home(home: impl AsRef<Path>) -> io::Result<Vec<SkillCatalogRecord>> {
    let installed = load_installed_skills(home.as_ref())?;
    let enabled_by_skill = load_enabled_skills_by_skill(home.as_ref())?;

    let mut catalog = installed
        .into_iter()
        .map(|record| {
            let enabled_by_agents = enabled_by_skill
                .get(&record.name)
                .map(|agents| agents.iter().cloned().collect::<Vec<_>>())
                .unwrap_or_default();
            SkillCatalogRecord {
                enabled_by_agent_count: enabled_by_agents.len(),
                enabled_by_agents,
                name: record.name,
                source_root: record.source_root,
                installed_root: record.installed_root,
            }
        })
        .collect::<Vec<_>>();

    catalog.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(catalog)
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

fn load_enabled_skills_by_skill(
    home: impl AsRef<Path>,
) -> io::Result<BTreeMap<String, BTreeSet<String>>> {
    let home = home.as_ref();
    let agents_dir = paths::runtime_home(home).join("agents");
    if !agents_dir.exists() {
        return Ok(BTreeMap::new());
    }

    let mut by_skill = BTreeMap::<String, BTreeSet<String>>::new();
    for entry in fs::read_dir(agents_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let Some(agent_name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };

        for enabled in load_enabled_skills(home, &agent_name)? {
            for skill_name in enabled.enabled {
                by_skill
                    .entry(skill_name)
                    .or_default()
                    .insert(enabled.agent_name.clone());
            }
        }
    }

    Ok(by_skill)
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

pub fn skills_inventory_response(surface: &SetupSurface, request_path: &str) -> HttpResponse {
    let agent_name =
        agent_name_from_request(request_path).unwrap_or_else(|| surface.current_agent_name());
    match skills_inventory_for_agent(surface.home(), &agent_name) {
        Ok(inventory) => {
            let body =
                serde_json::to_string_pretty(&inventory).expect("serialize skills inventory");
            HttpResponse::json(200, body)
        }
        Err(error) => HttpResponse::json(
            500,
            json!({ "error": format!("failed to load skills inventory: {error}") }).to_string(),
        ),
    }
}

pub fn skills_catalog_response(surface: &SetupSurface) -> HttpResponse {
    match skills_catalog_for_home(surface.home()) {
        Ok(catalog) => {
            let body = serde_json::to_string_pretty(&catalog).expect("serialize skills catalog");
            HttpResponse::json(200, body)
        }
        Err(error) => HttpResponse::json(
            500,
            json!({ "error": format!("failed to load skills catalog: {error}") }).to_string(),
        ),
    }
}

pub fn toggle_skill_response(surface: &SetupSurface, request: HttpRequest) -> HttpResponse {
    let Ok(change) = serde_json::from_slice::<EnableSkillChange>(&request.body) else {
        return HttpResponse::json(
            400,
            json!({ "error": "toggle payload must be valid JSON" }).to_string(),
        );
    };

    match set_skill_enabled(surface.home(), &change) {
        Ok(record) => {
            let body = serde_json::to_string_pretty(&record).expect("serialize enabled skills");
            HttpResponse::json(200, body)
        }
        Err(error) => HttpResponse::json(
            500,
            json!({ "error": format!("failed to update enabled skills: {error}") }).to_string(),
        ),
    }
}

fn agent_name_from_request(request_path: &str) -> Option<String> {
    let (_path, query) = request_path.split_once('?')?;
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=')?;
        if key == "agent" && !value.is_empty() {
            return Some(value.to_string());
        }
    }

    None
}
