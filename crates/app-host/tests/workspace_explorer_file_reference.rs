use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use matrixclaw_app_host::http::workspace_api::{workspace_surface_for_home, WorkspaceEntryKind};

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!("matrixclaw-home-{}-{}", std::process::id(), nanos));
    fs::create_dir_all(&home).expect("create temp home");
    home
}

fn temp_workspace(home: &Path) -> PathBuf {
    let root = home.join(".matrixclaw").join("config").join("workspace");
    fs::create_dir_all(root.join("notes").join("2026")).expect("create notes tree");
    fs::create_dir_all(root.join("projects").join("alpha")).expect("create project tree");
    fs::write(
        root.join("notes").join("2026").join("todo.md"),
        "# todo\n\nKeep this file unchanged.\n",
    )
    .expect("write nested note");
    fs::write(
        root.join("projects").join("alpha").join("README.md"),
        "# alpha\n",
    )
    .expect("write project readme");
    fs::write(root.join("prompt.md"), "Use workspace references only.\n").expect("write prompt");
    root
}

#[test]
fn workspace_explorer_file_reference() {
    let home = temp_home();
    let workspace_root = temp_workspace(&home);
    let before = fs::read_to_string(workspace_root.join("notes").join("2026").join("todo.md"))
        .expect("read nested workspace file before listing");
    let surface =
        workspace_surface_for_home(&home).expect("build workspace explorer surface for home");

    assert_eq!(
        surface.workspace_root(),
        workspace_root,
        "workspace surface should resolve to the configured workspace root"
    );

    let entries = surface
        .list_entries()
        .expect("list workspace entries from the backend contract");

    assert!(
        entries
            .iter()
            .any(|entry| entry.relative_path == PathBuf::from("notes/2026/todo.md")),
        "workspace listing should include nested files for prompt-safe selection"
    );

    let nested_reference = surface.reference_token_for_path("notes/2026/todo.md");
    let nested_reference_again = surface.reference_token_for_path("notes/2026/todo.md");
    assert_eq!(
        nested_reference, nested_reference_again,
        "selected file references must be stable across repeated requests"
    );

    assert!(
        entries
            .iter()
            .any(|entry| entry.kind == WorkspaceEntryKind::File
                && entry.reference_token == nested_reference),
        "workspace entries should expose a stable reference token for file insertion"
    );

    let after = fs::read_to_string(workspace_root.join("notes").join("2026").join("todo.md"))
        .expect("read nested workspace file after listing");
    assert_eq!(
        after, before,
        "listing and reference generation must not mutate file contents"
    );
}
