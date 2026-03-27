use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_app_host::install::desired_install_dir;

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!("matrixclaw-home-{}-{}", std::process::id(), nanos));
    fs::create_dir_all(&home).expect("create temp home");
    home
}

#[test]
fn install_without_privileged_writes() {
    let home = temp_home();
    let expected_bin_dir = desired_install_dir(&home);
    let expected_bin = expected_bin_dir.join("matrixclaw");
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate parent")
        .parent()
        .expect("workspace parent")
        .to_path_buf();
    let script = repo_root.join("scripts/install.sh");
    let built_binary = env!("CARGO_BIN_EXE_matrixclaw");

    let output = Command::new("sh")
        .arg(script)
        .env("HOME", &home)
        .env("MATRIXCLAW_SOURCE_BIN", built_binary)
        .output()
        .expect("run installer");

    assert!(
        output.status.success(),
        "installer failed unexpectedly: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        expected_bin.exists(),
        "expected installer to place the binary at {:?}, stderr: {}",
        expected_bin,
        String::from_utf8_lossy(&output.stderr)
    );

    let version_output = Command::new(&expected_bin)
        .arg("version")
        .output()
        .expect("run installed binary");

    assert!(
        version_output.status.success(),
        "installed binary did not run successfully: {}",
        String::from_utf8_lossy(&version_output.stderr)
    );
    let stdout = String::from_utf8_lossy(&version_output.stdout);
    assert!(
        stdout.contains("MatrixClaw 0.1.0"),
        "unexpected version output: {stdout}"
    );
}
