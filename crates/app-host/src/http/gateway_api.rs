use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::agent_store::load_agent_profiles;
use crate::http::{HttpResponse, SetupSurface};
use crate::paths;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatewayCatalogRecord {
    pub name: String,
    pub health: String,
    pub enabled_by_agent_count: usize,
}

pub const GATEWAY_CATALOG_ROUTE: &str = "/api/gateway";

pub fn is_gateway_catalog_route(path: &str) -> bool {
    crate::http::routes::normalize_path(path) == GATEWAY_CATALOG_ROUTE
}

pub fn gateway_catalog_path(home: impl AsRef<Path>) -> PathBuf {
    paths::runtime_home(home)
        .join("state")
        .join("catalogs")
        .join("gateway-catalog.json")
}

pub fn gateway_catalog_for_home(home: impl AsRef<Path>) -> io::Result<Vec<GatewayCatalogRecord>> {
    let mut catalog =
        load_snapshot_or_defaults(gateway_catalog_path(&home), seeded_gateway_catalog())?;
    let counts = count_enabled_gateways(home.as_ref())?;

    for record in &mut catalog {
        record.enabled_by_agent_count = counts.get(&record.name).copied().unwrap_or(0);
    }

    catalog.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(catalog)
}

pub fn gateway_catalog_response(surface: &SetupSurface) -> HttpResponse {
    match gateway_catalog_for_home(surface.home()) {
        Ok(catalog) => {
            let body = serde_json::to_string_pretty(&catalog).expect("serialize gateway catalog");
            HttpResponse::json(200, body)
        }
        Err(error) => HttpResponse::json(
            500,
            json!({ "error": format!("failed to load gateway catalog: {error}") }).to_string(),
        ),
    }
}

fn seeded_gateway_catalog() -> Vec<GatewayCatalogRecord> {
    vec![
        GatewayCatalogRecord {
            name: "matrix".to_string(),
            health: "healthy".to_string(),
            enabled_by_agent_count: 0,
        },
        GatewayCatalogRecord {
            name: "openclaw".to_string(),
            health: "healthy".to_string(),
            enabled_by_agent_count: 0,
        },
    ]
}

fn load_snapshot_or_defaults<T>(path: PathBuf, defaults: Vec<T>) -> io::Result<Vec<T>>
where
    T: DeserializeOwned,
{
    if !path.exists() {
        return Ok(defaults);
    }

    let body = fs::read_to_string(path)?;
    serde_json::from_str(&body).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn count_enabled_gateways(home: impl AsRef<Path>) -> io::Result<BTreeMap<String, usize>> {
    let mut counts = BTreeMap::<String, usize>::new();
    for agent in load_agent_profiles(home)? {
        for gateway in agent.enabled_gateways.into_iter().collect::<BTreeSet<_>>() {
            *counts.entry(gateway).or_insert(0) += 1;
        }
    }

    Ok(counts)
}
