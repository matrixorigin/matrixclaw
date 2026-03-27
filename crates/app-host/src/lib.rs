pub mod asset_manifest;
pub mod assets;
pub mod commands;
pub mod compat_registry;
pub mod execution;
pub mod install;
pub mod local_command;
pub mod paths;
pub mod plugin_launcher;
pub mod sandbox_backend;
pub mod setup;

pub const VERSION: &str = "0.1.0";

pub fn run(args: impl IntoIterator<Item = String>) -> i32 {
    let mut args = args.into_iter();
    let _ = args.next();
    match args.next().as_deref() {
        Some("version") => {
            println!("MatrixClaw {}", VERSION);
            0
        }
        None => match setup::ensure_first_launch() {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("setup failed: {error}");
                1
            }
        },
        _ => {
            eprintln!("usage: matrixclaw version");
            1
        }
    }
}
