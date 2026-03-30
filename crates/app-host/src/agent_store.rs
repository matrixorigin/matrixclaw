use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::paths;
use crate::session_binding_store::load_session_bindings;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentProfile {
    pub agent_name: String,
    pub title: String,
    pub crown_job: String,
    pub memory_summary: String,
    pub memory_signal_count: usize,
    pub pinned_memory_count: usize,
    pub enabled_skills: Vec<String>,
    pub enabled_mcp_servers: Vec<String>,
    pub enabled_gateways: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentProfileSummary {
    pub agent_name: String,
    pub title: String,
    pub crown_job: String,
    pub memory_summary: String,
    pub memory_signal_count: usize,
    pub pinned_memory_count: usize,
    pub enabled_skills: Vec<String>,
    pub enabled_mcp_servers: Vec<String>,
    pub enabled_gateways: Vec<String>,
    pub binding_count: usize,
}

pub fn agent_profile_path(home: impl AsRef<Path>, agent_name: impl AsRef<str>) -> PathBuf {
    paths::runtime_home(home)
        .join("agents")
        .join(agent_name.as_ref())
        .join("profile.json")
}

pub fn list_agent_profiles(home: impl AsRef<Path>) -> io::Result<Vec<AgentProfileSummary>> {
    let home = home.as_ref();
    let root = agent_profiles_dir(home);
    if !root.exists() {
        return Ok(Vec::new());
    }

    let bindings = load_session_bindings(home)?;
    let mut binding_counts = std::collections::HashMap::<String, usize>::new();
    for binding in bindings {
        *binding_counts.entry(binding.agent_name).or_insert(0) += 1;
    }

    let mut profiles = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let profile_path = entry.path().join("profile.json");
        if !profile_path.exists() {
            continue;
        }

        let body = fs::read_to_string(&profile_path)?;
        let profile: AgentProfile = serde_json::from_str(&body)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let binding_count = binding_counts
            .get(&profile.agent_name)
            .copied()
            .unwrap_or(0);
        profiles.push(AgentProfileSummary::from((profile, binding_count)));
    }

    profiles.sort_by(|left, right| left.agent_name.cmp(&right.agent_name));
    Ok(profiles)
}

pub fn save_agent_profile(
    home: impl AsRef<Path>,
    profile: &AgentProfile,
) -> io::Result<PathBuf> {
    let path = agent_profile_path(home, &profile.agent_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let body = serde_json::to_string_pretty(profile)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let temp_path = path.with_extension(format!("json.tmp-{}", temp_suffix()));
    fs::write(&temp_path, body)?;
    if path.exists() {
        fs::remove_file(&path)?;
    }
    fs::rename(&temp_path, &path)?;
    Ok(path)
}

impl From<(AgentProfile, usize)> for AgentProfileSummary {
    fn from((profile, binding_count): (AgentProfile, usize)) -> Self {
        Self {
            agent_name: profile.agent_name,
            title: profile.title,
            crown_job: profile.crown_job,
            memory_summary: profile.memory_summary,
            memory_signal_count: profile.memory_signal_count,
            pinned_memory_count: profile.pinned_memory_count,
            enabled_skills: profile.enabled_skills,
            enabled_mcp_servers: profile.enabled_mcp_servers,
            enabled_gateways: profile.enabled_gateways,
            binding_count,
        }
    }
}

fn agent_profiles_dir(home: impl AsRef<Path>) -> PathBuf {
    paths::runtime_home(home).join("agents")
}

fn temp_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    format!("{}-{nanos}", std::process::id())
}
