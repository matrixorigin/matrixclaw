use std::path::{Path, PathBuf};

pub fn desired_install_dir(home: impl AsRef<Path>) -> PathBuf {
    home.as_ref().join(".matrixclaw").join("bin")
}
