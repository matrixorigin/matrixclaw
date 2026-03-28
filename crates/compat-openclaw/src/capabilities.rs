use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CapabilityDescriptor {
    pub protocol: &'static str,
    pub version: &'static str,
    pub agent_listing_supported: bool,
}

impl Default for CapabilityDescriptor {
    fn default() -> Self {
        Self {
            protocol: "openclaw",
            version: "0.1",
            agent_listing_supported: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentDescriptor {
    pub id: String,
    pub name: String,
}
