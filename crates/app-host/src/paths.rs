use std::env;
use std::path::{Path, PathBuf};

pub fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("resolve current directory"))
}

pub fn runtime_home(home: impl AsRef<Path>) -> PathBuf {
    home.as_ref().join(".matrixclaw")
}

pub fn managed_assets_dir(home: impl AsRef<Path>) -> PathBuf {
    runtime_home(home).join("assets")
}

pub fn config_dir(home: impl AsRef<Path>) -> PathBuf {
    runtime_home(home).join("config")
}

pub fn config_path(home: impl AsRef<Path>) -> PathBuf {
    config_dir(home).join("config.json")
}
