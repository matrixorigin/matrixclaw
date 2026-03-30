use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_app_host::http::SetupSurface;
use matrixclaw_app_host::server::spawn_test_server;
use matrixclaw_app_host::ui_assets::{UiAssetLayout, UI_ENTRY_HTML, UI_WORKSPACE_DIR};

#[test]
fn health_probe_accepts_desktop_shell_origin() {
    let home = temp_home();
    let repo_root = temp_repo_root();
    let surface = SetupSurface::new(&home, UiAssetLayout::from_repo_root(repo_root));
    let server = spawn_test_server(surface).expect("spawn setup surface");

    let mut stream = TcpStream::connect(server.address).expect("connect test server");
    stream
        .write_all(
            concat!(
                "GET /healthz HTTP/1.1\r\n",
                "Host: 127.0.0.1\r\n",
                "Origin: tauri://localhost\r\n",
                "Connection: close\r\n",
                "\r\n",
            )
            .as_bytes(),
        )
        .expect("send health probe");

    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .expect("read health probe response");
    let response = String::from_utf8(response).expect("response should be utf-8");
    let normalized = response.to_ascii_lowercase();

    assert!(
        normalized.starts_with("http/1.1 200"),
        "health probe should succeed: {response}"
    );
    assert!(
        normalized.contains("access-control-allow-origin: tauri://localhost"),
        "health probe should allow desktop shell origin: {response}"
    );
    assert!(
        normalized.contains("vary: origin"),
        "health probe should vary by origin: {response}"
    );

    server.shutdown().expect("shutdown setup surface");
}

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!(
        "matrixclaw-desktop-shell-health-cors-home-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&home).expect("create temp home");
    home
}

fn temp_repo_root() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let root = env::temp_dir().join(format!(
        "matrixclaw-desktop-shell-health-cors-ui-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(root.join(UI_WORKSPACE_DIR).join("build").join("_app"))
        .expect("create fixture build tree");
    fs::write(
        root.join(UI_WORKSPACE_DIR)
            .join("build")
            .join(UI_ENTRY_HTML),
        "<html><body>setup shell</body></html>",
    )
    .expect("write setup shell");
    root
}
