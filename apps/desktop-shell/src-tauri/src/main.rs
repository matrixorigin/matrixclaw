#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;

use tauri::Manager;

const UI_BUILD_RESOURCE_DIR: &str = "ui/build";

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let home = matrixclaw_app_host::paths::home_dir();

            if let Some(source_dir) = resolve_bundled_ui_source_dir(app) {
                stage_bundled_ui_assets(&source_dir, &home)?;
            }

            spawn_embedded_runtime(home);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("launch MatrixClaw desktop shell");
}

fn resolve_bundled_ui_source_dir(app: &tauri::App) -> Option<PathBuf> {
    if let Some(build_dir) = std::env::var_os("MATRIXCLAW_UI_BUILD_DIR") {
        let build_dir = PathBuf::from(build_dir);
        if build_dir.join("index.html").is_file() {
            return Some(build_dir);
        }
    }

    let Ok(resource_dir) = app.path().resource_dir() else {
        return None;
    };

    let bundled = resource_dir.join(UI_BUILD_RESOURCE_DIR);
    bundled.join("index.html").is_file().then_some(bundled)
}

fn spawn_embedded_runtime(home: PathBuf) {
    thread::spawn(move || {
        if let Err(error) = matrixclaw_app_host::server::serve_for_home(&home) {
            if error.kind() != io::ErrorKind::AddrInUse {
                eprintln!("matrixclaw desktop runtime failed: {error}");
            }
        }
    });
}

fn stage_bundled_ui_assets(source_dir: &Path, home: &Path) -> io::Result<PathBuf> {
    let target_dir = matrixclaw_app_host::paths::managed_assets_dir(home)
        .join("ui")
        .join("build");
    copy_dir_recursive(source_dir, &target_dir)?;
    Ok(target_dir)
}

fn copy_dir_recursive(source: &Path, target: &Path) -> io::Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else if file_type.is_file() {
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{copy_dir_recursive, stage_bundled_ui_assets};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "matrixclaw-desktop-shell-{}-{}-{}",
            label,
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn copy_dir_recursive_preserves_nested_files() {
        let source = temp_dir("source");
        let target = temp_dir("target");
        fs::create_dir_all(source.join("_app").join("immutable")).expect("create source tree");
        fs::write(source.join("index.html"), "<html>bundled shell</html>").expect("write shell");
        fs::write(
            source.join("_app").join("immutable").join("app.js"),
            "console.log('matrixclaw');",
        )
        .expect("write js");

        copy_dir_recursive(&source, &target).expect("copy dir");

        assert!(target.join("index.html").is_file());
        assert!(target.join("_app").join("immutable").join("app.js").is_file());
    }

    #[test]
    fn stage_bundled_ui_assets_copies_into_runtime_home() {
        let source = temp_dir("ui-build");
        let home = temp_dir("home");
        fs::create_dir_all(source.join("_app")).expect("create source tree");
        fs::write(source.join("index.html"), "<html>bundled shell</html>").expect("write shell");
        fs::write(source.join("setup.html"), "<html>setup</html>").expect("write setup");
        fs::write(source.join("_app").join("app.js"), "console.log('matrixclaw');")
            .expect("write js");

        let staged = stage_bundled_ui_assets(&source, &home).expect("stage bundled ui assets");
        assert_eq!(
            staged,
            matrixclaw_app_host::paths::managed_assets_dir(&home)
                .join("ui")
                .join("build")
        );
        assert!(staged.join("index.html").is_file());
        assert!(staged.join("setup.html").is_file());
        assert!(staged.join("_app").join("app.js").is_file());
    }
}
