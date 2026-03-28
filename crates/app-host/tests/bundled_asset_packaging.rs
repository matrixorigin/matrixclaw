use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_app_host::http::{setup_surface_for_home, HttpRequest};
use matrixclaw_app_host::paths;
use matrixclaw_app_host::ui_assets::{
    UiAssetLayout, UI_BUILD_DIR, UI_ENTRY_HTML, UI_SETUP_HTML, UI_SKILLS_HTML, UI_WORKSPACE_DIR,
    UI_WORKSPACE_HTML,
};

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!(
        "matrixclaw-bundled-home-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&home).expect("create temp home");
    home
}

fn seed_bundled_ui(home: &PathBuf) -> UiAssetLayout {
    let build_dir = paths::managed_assets_dir(home)
        .join(UI_WORKSPACE_DIR)
        .join(UI_BUILD_DIR);
    fs::create_dir_all(build_dir.join("_app")).expect("create bundled build dir");
    fs::write(
        build_dir.join(UI_ENTRY_HTML),
        "<html><body>bundled shell</body></html>",
    )
    .expect("write shell html");
    fs::write(
        build_dir.join(UI_SETUP_HTML),
        "<html><body>bundled setup</body></html>",
    )
    .expect("write setup html");
    fs::write(
        build_dir.join(UI_WORKSPACE_HTML),
        "<html><body>bundled workspace</body></html>",
    )
    .expect("write workspace html");
    fs::write(
        build_dir.join(UI_SKILLS_HTML),
        "<html><body>bundled skills</body></html>",
    )
    .expect("write skills html");
    fs::write(
        build_dir.join("_app").join("app.js"),
        "console.log('matrixclaw');",
    )
    .expect("write bundled app js");
    UiAssetLayout::bundled_for_home(home)
}

#[test]
fn bundled_asset_packaging() {
    let home = temp_home();
    let bundled = seed_bundled_ui(&home);
    env::remove_var("MATRIXCLAW_REPO_ROOT");
    env::remove_var("MATRIXCLAW_UI_BUILD_DIR");

    let discovered = UiAssetLayout::discover_for_home(&home);
    assert_eq!(
        discovered.build_dir, bundled.build_dir,
        "discover_for_home should prefer bundled UI assets inside the runtime home"
    );

    let surface = setup_surface_for_home(&home).expect("create setup surface");

    let setup = surface.handle(HttpRequest::get("/setup"));
    assert_eq!(setup.status_code, 200);
    assert!(
        setup.body_text().contains("bundled setup"),
        "setup route should be served from bundled assets"
    );

    let workspace = surface.handle(HttpRequest::get("/workspace"));
    assert_eq!(workspace.status_code, 200);
    assert!(
        workspace.body_text().contains("bundled workspace"),
        "workspace route should be served from bundled assets"
    );

    let static_asset = surface.handle(HttpRequest::get("/_app/app.js"));
    assert_eq!(static_asset.status_code, 200);
    assert!(
        static_asset.body_text().contains("matrixclaw"),
        "static bundled assets should also resolve from the runtime home"
    );
}
