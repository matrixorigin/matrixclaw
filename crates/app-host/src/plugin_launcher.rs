use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginLaunchRequest {
    pub adapter_path: PathBuf,
    pub manifest_path: PathBuf,
    pub installed_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginLaunchOutcome {
    pub adapter_command: String,
    pub adapter_invoked: bool,
    pub exposed_capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PluginAdapterContract {
    schema_version: String,
    manifest_path: PathBuf,
    installed_root: PathBuf,
    adapter_command: String,
    adapter_args: Vec<String>,
    capabilities: Vec<String>,
    support_tier: String,
}

pub fn launch_plugin_via_adapter(request: &PluginLaunchRequest) -> io::Result<PluginLaunchOutcome> {
    let contract = read_adapter_contract(&request.adapter_path)?;

    if !request.manifest_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "plugin manifest not found at {}",
                request.manifest_path.display()
            ),
        ));
    }

    if contract.schema_version != "1" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported adapter contract version {}",
                contract.schema_version
            ),
        ));
    }

    if contract.manifest_path != request.manifest_path {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "adapter contract manifest path does not match request",
        ));
    }

    if contract.installed_root != request.installed_root {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "adapter contract installed root does not match request",
        ));
    }

    if contract.adapter_command.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "adapter command is empty",
        ));
    }

    Ok(PluginLaunchOutcome {
        adapter_command: contract.adapter_command,
        adapter_invoked: true,
        exposed_capabilities: contract.capabilities,
    })
}

fn read_adapter_contract(path: &PathBuf) -> io::Result<PluginAdapterContract> {
    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}
