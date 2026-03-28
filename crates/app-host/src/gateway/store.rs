use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GatewaySessionStore {
    pub matrix_threads: Vec<MatrixThreadBinding>,
    pub processed_inbound_event_ids: Vec<String>,
    pub pending_delivery_retries: Vec<GatewayDeliveryRetryRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatrixThreadBinding {
    pub room_id: String,
    pub thread_id: Option<String>,
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatewayDeliveryRetryRecord {
    pub gateway_kind: String,
    pub channel_id: String,
    pub thread_id: Option<String>,
    pub reply_to: Option<String>,
    pub body: String,
}

impl GatewaySessionStore {
    pub fn load_or_default(home: impl AsRef<Path>) -> io::Result<Self> {
        let path = store_path(home);
        if !path.exists() {
            return Ok(Self::default());
        }

        let body = fs::read_to_string(path)?;
        serde_json::from_str(&body)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }

    pub fn save(&self, home: impl AsRef<Path>) -> io::Result<PathBuf> {
        let path = store_path(home);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let body = serde_json::to_string_pretty(self)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        fs::write(&path, body)?;
        Ok(path)
    }

    pub fn bind_matrix_thread(
        &mut self,
        room_id: impl Into<String>,
        thread_id: Option<&str>,
        session_id: impl Into<String>,
    ) -> Result<(), String> {
        let room_id = room_id.into();
        let session_id = session_id.into();
        if room_id.trim().is_empty() {
            return Err("room_id is required".to_string());
        }
        if session_id.trim().is_empty() {
            return Err("session_id is required".to_string());
        }

        let thread_id = normalized_optional(thread_id);
        if let Some(existing) = self
            .matrix_threads
            .iter_mut()
            .find(|binding| binding.room_id == room_id && binding.thread_id == thread_id)
        {
            existing.session_id = session_id;
            return Ok(());
        }

        self.matrix_threads.push(MatrixThreadBinding {
            room_id,
            thread_id,
            session_id,
        });
        Ok(())
    }

    pub fn resolve_matrix_thread(
        &self,
        room_id: &str,
        thread_id: Option<&str>,
    ) -> Option<&str> {
        let thread_id = normalized_optional(thread_id);
        self.matrix_threads
            .iter()
            .find(|binding| binding.room_id == room_id && binding.thread_id == thread_id)
            .map(|binding| binding.session_id.as_str())
    }
}

fn store_path(home: impl AsRef<Path>) -> PathBuf {
    home.as_ref()
        .join(".matrixclaw")
        .join("state")
        .join("gateways")
        .join("gateway-session-store.json")
}

fn normalized_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}
