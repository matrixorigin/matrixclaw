use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_app_host::agent_store::{agent_profile_path, list_agent_profiles, AgentProfile};
use matrixclaw_app_host::session_binding_store::bind_session_to_agent;

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!("matrixclaw-home-{}-{}", std::process::id(), nanos));
    fs::create_dir_all(&home).expect("create temp home");
    home
}

fn seeded_home_with_agents() -> PathBuf {
    let home = temp_home();
    let profile = AgentProfile {
        agent_name: "atlas".to_string(),
        title: "Atlas".to_string(),
        crown_job: "Research topics and synthesize findings.".to_string(),
        memory_summary: "Keeps long-running research context.".to_string(),
        memory_signal_count: 14,
        pinned_memory_count: 3,
        enabled_skills: vec!["web_search".to_string(), "summarize".to_string()],
        enabled_mcp_servers: vec!["search-01".to_string()],
        enabled_gateways: vec!["matrix".to_string()],
    };

    let path = agent_profile_path(&home, &profile.agent_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create agent profile directory");
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(&profile).expect("serialize profile"),
    )
    .expect("write agent profile");

    bind_session_to_agent(&home, "atlas-session-1", "atlas").expect("seed first binding");
    bind_session_to_agent(&home, "atlas-session-2", "atlas").expect("seed second binding");
    bind_session_to_agent(&home, "beta-session-1", "beta").expect("seed unrelated binding");

    home
}

#[test]
fn agent_directory_contract_lists_profiles_with_binding_counts() {
    let home = seeded_home_with_agents();
    let agents = list_agent_profiles(&home).expect("load agent profiles");

    let atlas = agents
        .iter()
        .find(|agent| agent.agent_name == "atlas")
        .expect("atlas profile");
    assert_eq!(atlas.binding_count, 2);
    assert!(
        atlas.enabled_skills.contains(&"web_search".to_string()),
        "profile summary should preserve enabled skills"
    );
}
