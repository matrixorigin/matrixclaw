use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::provenance::ImportProvenance;
use crate::skill_manifest::SupportTier;

pub const PLUGIN_ENTRY_NAME: &str = "openclaw.plugin.json";
pub const NORMALIZED_MANIFEST_NAME: &str = "matrixclaw.plugin.json";
pub const ADAPTER_CONTRACT_NAME: &str = "matrixclaw.plugin.adapter.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginInstallReasonCode {
    InProcessExtension,
    BridgeRuntimeRequired,
    UnsupportedTransport,
    UnsupportedArtifact,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginInstallDiagnostic {
    pub code: PluginInstallReasonCode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginDetection {
    pub id: String,
    pub name: String,
    pub root: PathBuf,
    pub entry: PathBuf,
    pub description: String,
    pub origin: String,
    pub kind: String,
    pub transport: String,
    pub tier: SupportTier,
    pub capabilities: Vec<String>,
    pub compatibility_note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginInstallOutcome {
    Imported {
        manifest_path: PathBuf,
        installed_root: PathBuf,
        provenance_path: PathBuf,
        adapter_path: PathBuf,
    },
    Rejected {
        reason: String,
    },
}

impl PluginInstallOutcome {
    pub fn rejection_diagnostic(&self) -> Option<PluginInstallDiagnostic> {
        match self {
            PluginInstallOutcome::Imported { .. } => None,
            PluginInstallOutcome::Rejected { reason } => Some(diagnostic_from_reason(reason)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct OpenClawPluginManifest {
    name: Option<String>,
    description: Option<String>,
    kind: Option<String>,
    transport: Option<OpenClawTransport>,
    capabilities: Option<OpenClawCapabilities>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct OpenClawTransport {
    #[serde(rename = "type")]
    kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct OpenClawCapabilities {
    provides: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PackageJsonManifest {
    #[serde(rename = "type")]
    package_type: Option<String>,
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

pub fn detect_plugin_root(root: impl AsRef<Path>) -> io::Result<Option<PluginDetection>> {
    let root = root.as_ref();
    let entry = root.join(PLUGIN_ENTRY_NAME);
    if !entry.exists() {
        return Ok(None);
    }

    let manifest = read_plugin_manifest(&entry)?;
    let transport = manifest
        .transport
        .as_ref()
        .and_then(|value| value.kind.as_deref())
        .unwrap_or("jsonrpc_stdio")
        .to_string();
    let tier = classify_transport(&transport);
    let name = manifest
        .name
        .clone()
        .or_else(|| {
            root.file_name()
                .and_then(|value| value.to_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "plugin".to_string());
    let description = manifest
        .description
        .clone()
        .unwrap_or_else(|| format!("Imported plugin package {name}"));
    let kind = manifest
        .kind
        .clone()
        .unwrap_or_else(|| "provider".to_string());
    let capabilities = manifest
        .capabilities
        .map(|value| value.provides)
        .unwrap_or_default();
    let compatibility_note = classify_compatibility_note(root, &name, &transport, &tier)?;

    Ok(Some(PluginDetection {
        id: name.clone(),
        name,
        root: root.to_path_buf(),
        entry,
        description,
        origin: "openclaw".to_string(),
        kind,
        transport,
        tier,
        capabilities,
        compatibility_note,
    }))
}

pub fn normalize_plugin_manifest(detection: &PluginDetection) -> serde_json::Value {
    let tier = match detection.tier {
        SupportTier::Native => "native",
        SupportTier::Shimmed => "shimmed",
        SupportTier::BridgeOnly => "bridge_only",
        SupportTier::Unsupported => "unsupported",
    };
    let root = detection.root.clone();
    let id = detection.id.clone();
    let name = detection.name.clone();
    let kind = detection.kind.clone();
    let description = detection.description.clone();
    let transport = detection.transport.clone();
    let origin = detection.origin.clone();
    let capabilities = detection.capabilities.clone();

    serde_json::json!({
        "schemaVersion": "1",
        "id": id,
        "name": name,
        "kind": kind,
        "description": description,
        "entrypoint": {
            "command": "matrixclaw-plugin-adapter",
            "args": ["--manifest", PLUGIN_ENTRY_NAME],
        },
        "transport": {
            "type": transport,
        },
        "capabilities": {
            "provides": capabilities,
            "consumes": [],
        },
        "compat": {
            "origin": origin,
            "artifactClass": "plugin_process",
            "tier": tier,
            "importMode": "adapter",
            "originalManifest": PLUGIN_ENTRY_NAME,
        },
        "source": {
            "type": "local_path",
            "ref": root,
            "path": root,
        },
        "provenance": {
            "originalFiles": [PLUGIN_ENTRY_NAME],
        }
    })
}

pub fn install_plugin_package(
    source_root: impl AsRef<Path>,
    runtime_home: impl AsRef<Path>,
) -> io::Result<PluginInstallOutcome> {
    let detection = match detect_plugin_root(&source_root)? {
        Some(detection) => detection,
        None => {
            return Ok(PluginInstallOutcome::Rejected {
                reason: format!("missing {PLUGIN_ENTRY_NAME}"),
            });
        }
    };

    if let Some(reason) = detection.compatibility_note.clone() {
        return Ok(PluginInstallOutcome::Rejected { reason });
    }

    let runtime_home = runtime_home.as_ref();
    let installed_root = runtime_home
        .join(".matrixclaw")
        .join("plugins")
        .join(&detection.id);
    let manifest_path = installed_root.join(NORMALIZED_MANIFEST_NAME);
    let provenance_path = installed_root.join("provenance.json");
    let adapter_path = installed_root.join(ADAPTER_CONTRACT_NAME);
    let staging_root = staging_dir(&installed_root);

    prepare_staging_root(&staging_root)?;

    fs::create_dir_all(staging_root.parent().unwrap_or(runtime_home))?;
    fs::write(
        staging_root.join(PLUGIN_ENTRY_NAME),
        fs::read_to_string(&detection.entry)?,
    )?;

    let normalized = normalize_plugin_manifest(&detection);
    fs::write(
        staging_root.join(NORMALIZED_MANIFEST_NAME),
        serde_json::to_string_pretty(&normalized)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
    )?;

    let provenance = ImportProvenance::new(
        detection.origin.clone(),
        detection.root.clone(),
        installed_root.clone(),
        manifest_path.clone(),
        vec![PLUGIN_ENTRY_NAME.to_string()],
    )
    .with_generated_files(vec![
        NORMALIZED_MANIFEST_NAME.to_string(),
        ADAPTER_CONTRACT_NAME.to_string(),
    ])
    .with_compatibility("plugin_process", "shimmed")
    .with_installed_at_now();
    provenance.save_to(staging_root.join("provenance.json"))?;

    let contract = PluginAdapterContract {
        schema_version: "1".to_string(),
        manifest_path: manifest_path.clone(),
        installed_root: installed_root.clone(),
        adapter_command: "matrixclaw-plugin-adapter".to_string(),
        adapter_args: vec![
            "--manifest".to_string(),
            NORMALIZED_MANIFEST_NAME.to_string(),
        ],
        capabilities: detection.capabilities.clone(),
        support_tier: "shimmed".to_string(),
    };
    fs::write(
        staging_root.join(ADAPTER_CONTRACT_NAME),
        serde_json::to_string_pretty(&contract)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
    )?;

    if installed_root.exists() {
        fs::remove_dir_all(&installed_root)?;
    }
    fs::rename(&staging_root, &installed_root)?;

    Ok(PluginInstallOutcome::Imported {
        manifest_path,
        installed_root,
        provenance_path,
        adapter_path,
    })
}

fn classify_transport(transport: &str) -> SupportTier {
    match transport {
        "jsonrpc_stdio" | "mcp_stdio" | "mcp_http" => SupportTier::Shimmed,
        "bridge_runtime" => SupportTier::BridgeOnly,
        "inprocess" | "module" | "bun" | "node" => SupportTier::Unsupported,
        _ => SupportTier::Unsupported,
    }
}

fn classify_compatibility_note(
    root: &Path,
    name: &str,
    transport: &str,
    tier: &SupportTier,
) -> io::Result<Option<String>> {
    match tier {
        SupportTier::Shimmed => {
            if is_inprocess_extension(root)? {
                Ok(Some(format!(
                    "plugin {name} is an in-process OpenClaw extension and requires a bridge runtime or manual rewrite"
                )))
            } else {
                Ok(None)
            }
        }
        SupportTier::BridgeOnly => Ok(Some(format!(
            "plugin {name} requires a bridge runtime to install"
        ))),
        SupportTier::Unsupported => {
            if is_inprocess_extension(root)? {
                Ok(Some(format!(
                    "plugin {name} is an in-process OpenClaw extension and requires a bridge runtime or manual rewrite"
                )))
            } else if matches!(transport, "inprocess" | "module" | "bun" | "node") {
                Ok(Some(format!(
                    "plugin {name} uses unsupported transport {transport}"
                )))
            } else {
                Ok(Some(format!("plugin {name} is unsupported in MatrixClaw")))
            }
        }
        SupportTier::Native => Ok(None),
    }
}

fn is_inprocess_extension(root: &Path) -> io::Result<bool> {
    let package_json = root.join("package.json");
    if !package_json.exists() {
        return Ok(false);
    }

    let manifest = read_package_json(&package_json)?;
    if manifest.package_type.as_deref() != Some("module") {
        return Ok(false);
    }

    has_runtime_source_files(root)
}

fn read_package_json(path: &Path) -> io::Result<PackageJsonManifest> {
    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn has_runtime_source_files(root: &Path) -> io::Result<bool> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }

            let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
                continue;
            };

            if matches!(
                extension,
                "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" | "mts" | "cts"
            ) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn read_plugin_manifest(path: &Path) -> io::Result<OpenClawPluginManifest> {
    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn diagnostic_from_reason(reason: &str) -> PluginInstallDiagnostic {
    let normalized = reason.to_ascii_lowercase();
    let code = if normalized.contains("in-process") || normalized.contains("bun") {
        PluginInstallReasonCode::InProcessExtension
    } else if normalized.contains("bridge") {
        PluginInstallReasonCode::BridgeRuntimeRequired
    } else if normalized.contains("transport") {
        PluginInstallReasonCode::UnsupportedTransport
    } else if normalized.contains("artifact") {
        PluginInstallReasonCode::UnsupportedArtifact
    } else {
        PluginInstallReasonCode::Unknown
    };

    PluginInstallDiagnostic {
        code,
        message: reason.to_string(),
    }
}

fn staging_dir(installed_root: &Path) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let base = installed_root
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("plugin");
    installed_root.with_file_name(format!("{base}.staging-{}-{}", std::process::id(), nonce))
}

fn prepare_staging_root(staging_root: &Path) -> io::Result<()> {
    if staging_root.exists() {
        fs::remove_dir_all(staging_root)?;
    }
    fs::create_dir_all(staging_root)
}
