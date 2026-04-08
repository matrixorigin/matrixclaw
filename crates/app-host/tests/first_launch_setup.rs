use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use zstar_app_host::setup::config_path;

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
fn first_launch_setup() {
    let home = temp_home();
    let expected_config = config_path(&home);

    let output = Command::new(env!("CARGO_BIN_EXE_zstar"))
        .env("HOME", &home)
        .output()
        .expect("run zstar");

    assert!(
        !expected_config.exists(),
        "expected first launch to defer config persistence until setup completion at {:?}, stdout: {}, stderr: {}",
        expected_config,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("/setup"),
        "expected startup to announce the setup surface, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}
