use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::paths;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionAgentBinding {
    pub session_id: String,
    pub agent_name: String,
}

pub fn bind_session_to_agent(
    home: impl AsRef<Path>,
    session_id: impl AsRef<str>,
    agent_name: impl AsRef<str>,
) -> io::Result<SessionAgentBinding> {
    let session_id = trim_required(session_id.as_ref(), "session_id")?;
    let agent_name = trim_required(agent_name.as_ref(), "agent_name")?;

    let _guard = binding_write_lock().lock().expect("binding store lock");
    let mut bindings = load_session_bindings(home.as_ref())?;
    if let Some(existing) = bindings
        .iter()
        .find(|binding| binding.session_id == session_id)
    {
        if existing.agent_name != agent_name {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "session {session_id} already bound to {}",
                    existing.agent_name
                ),
            ));
        }

        return Ok(existing.clone());
    }

    let created = SessionAgentBinding {
        session_id,
        agent_name,
    };
    bindings.push(created.clone());
    save_session_bindings(home.as_ref(), &bindings)?;
    Ok(created)
}

pub fn load_session_bindings(home: impl AsRef<Path>) -> io::Result<Vec<SessionAgentBinding>> {
    let path = session_bindings_path(home);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let body = fs::read_to_string(path)?;
    serde_json::from_str(&body).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

pub fn save_session_bindings(
    home: impl AsRef<Path>,
    bindings: &[SessionAgentBinding],
) -> io::Result<PathBuf> {
    let path = session_bindings_path(home);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let body = serde_json::to_string_pretty(bindings)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let temp_path = path.with_extension(format!("json.tmp-{}", temp_suffix()));
    fs::write(&temp_path, body)?;
    replace_file(&temp_path, &path)?;
    Ok(path)
}

pub fn session_bindings_path(home: impl AsRef<Path>) -> PathBuf {
    paths::runtime_home(home)
        .join("state")
        .join("session-agent-bindings.json")
}

pub fn session_binding_for_session_id(
    home: impl AsRef<Path>,
    session_id: impl AsRef<str>,
) -> io::Result<Option<SessionAgentBinding>> {
    let session_id = trim_required(session_id.as_ref(), "session_id")?;
    Ok(load_session_bindings(home)?
        .into_iter()
        .find(|binding| binding.session_id == session_id))
}

fn trim_required(value: &str, field: &str) -> io::Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{field} is required"),
        ));
    }

    Ok(trimmed.to_string())
}

fn temp_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    format!("{}-{nanos}", std::process::id())
}

fn binding_write_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn replace_file(source: &Path, destination: &Path) -> io::Result<()> {
    if !destination.exists() {
        if let Err(error) = fs::rename(source, destination) {
            let _ = fs::remove_file(source);
            return Err(error);
        }
        return Ok(());
    }

    let backup_path = destination.with_extension(format!("json.bak-{}", temp_suffix()));
    fs::rename(destination, &backup_path)?;

    if let Err(error) = fs::rename(source, destination) {
        let _ = fs::rename(&backup_path, destination);
        let _ = fs::remove_file(source);
        return Err(error);
    }

    let _ = fs::remove_file(&backup_path);
    Ok(())
}
