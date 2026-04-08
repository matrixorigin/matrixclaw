use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use zstar_app_host::session_binding_store::bind_session_to_agent;

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!("zstar-home-{}-{}", std::process::id(), nanos));
    fs::create_dir_all(&home).expect("create temp home");
    home
}

#[test]
fn session_agent_binding_rejects_drift() {
    let home = temp_home();

    bind_session_to_agent(&home, "session-a", "atlas").expect("bind initial agent");

    let same = bind_session_to_agent(&home, "session-a", "atlas").expect("rebind same agent");
    assert_eq!(same.agent_name, "atlas");

    let error = bind_session_to_agent(&home, "session-a", "scribe")
        .expect_err("drifting a session to a new agent should fail");
    assert!(
        error.to_string().contains("already bound"),
        "error should mention the existing binding: {error}"
    );
}
