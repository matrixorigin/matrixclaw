use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_app_host::ui_assets::{UiAssetLayout, UI_ENTRY_HTML, UI_WORKSPACE_DIR};

fn temp_repo_root() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "matrixclaw-ui-fixture-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(root.join(UI_WORKSPACE_DIR).join("build").join("workspace"))
        .expect("create ui fixture");
    fs::write(
        root.join(UI_WORKSPACE_DIR)
            .join("build")
            .join(UI_ENTRY_HTML),
        "<html><body>matrixclaw shell</body></html>",
    )
    .expect("write ui entry html");
    fs::write(
        root.join(UI_WORKSPACE_DIR)
            .join("build")
            .join("workspace")
            .join("index.html"),
        "<html><body>workspace route</body></html>",
    )
    .expect("write ui workspace html");
    root
}

#[test]
fn embedded_ui_assets() {
    let repo_root = temp_repo_root();
    let layout = UiAssetLayout::from_repo_root(&repo_root);

    let shell = layout
        .shell_document_for_route("/")
        .expect("the root route should resolve to the shell document");
    assert_eq!(shell, layout.entry_html());
    assert!(
        shell.exists(),
        "fixture-backed shell document should exist at {shell:?}"
    );

    let client_route = layout.shell_document_for_route("/workspace");
    assert!(
        client_route.is_some(),
        "browser refresh on /workspace should still resolve through the shell"
    );
    let client_route = client_route.expect("workspace route should reuse the shell");
    assert_eq!(
        client_route,
        layout.entry_html(),
        "SPA fallback should map client-side routes to the shell entry"
    );
}
