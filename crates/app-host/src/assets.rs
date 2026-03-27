use std::io;
use std::path::{Path, PathBuf};

use crate::asset_manifest::BrowserAssetManifest;
use crate::paths;

pub fn managed_assets_dir(home: impl AsRef<Path>) -> PathBuf {
    paths::managed_assets_dir(home)
}

pub fn browser_assets_dir(home: impl AsRef<Path>) -> PathBuf {
    managed_assets_dir(home).join("browser")
}

pub fn browser_asset_manifest_path(home: impl AsRef<Path>) -> PathBuf {
    BrowserAssetManifest::manifest_path(home)
}

pub trait BrowserAssetDownloader {
    fn download_browser_asset(&mut self, home: &Path) -> io::Result<BrowserAssetManifest>;
}

pub fn browser_asset_is_cached(home: impl AsRef<Path>) -> bool {
    browser_asset_manifest_path(home).exists()
}

pub fn ensure_browser_asset(
    home: impl AsRef<Path>,
    downloader: &mut impl BrowserAssetDownloader,
) -> io::Result<BrowserAssetManifest> {
    if let Some(manifest) = BrowserAssetManifest::load_from_home(&home)? {
        return Ok(manifest);
    }

    let manifest = downloader.download_browser_asset(home.as_ref())?;
    let _ = manifest.save_to_home(&home)?;
    Ok(manifest)
}
