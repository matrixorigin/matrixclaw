use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn optional_matrix_gateway_startup() {
    let home = temp_home();

    let output = Command::new(env!("CARGO_BIN_EXE_zstar"))
        .env("HOME", &home)
        .output()
        .expect("run zstar");

    assert!(
        output.status.success(),
        "startup should still succeed without Matrix gateway configuration: stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("ZStar setup available"),
        "expected normal startup output to remain available, stdout: {stdout}"
    );
    assert!(
        stdout.contains("Matrix gateway disabled"),
        "expected startup to clearly report that the Matrix gateway is optional and not active, stdout: {stdout}"
    );
}

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!(
        "zstar-matrix-gateway-home-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&home).expect("create temp home");
    home
}
