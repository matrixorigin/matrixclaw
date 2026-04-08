use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use zstar_app_host::asset_manifest::BrowserAssetManifest;
use zstar_app_host::assets;

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    let home = env::temp_dir().join(format!("zstar-home-{}-{}", std::process::id(), nanos));
    fs::create_dir_all(&home).expect("create temp home");
    home
}

#[derive(Default)]
struct FakeDownloadService {
    downloads: usize,
}

impl FakeDownloadService {
    fn download_browser_asset(&mut self, home: &Path) -> BrowserAssetManifest {
        self.downloads += 1;
        BrowserAssetManifest::new(
            "browser",
            "1.0.0",
            assets::browser_asset_manifest_path(home),
        )
    }
}

impl assets::BrowserAssetDownloader for FakeDownloadService {
    fn download_browser_asset(&mut self, home: &Path) -> std::io::Result<BrowserAssetManifest> {
        Ok(self.download_browser_asset(home))
    }
}

struct BrowserCapabilityStub<'a> {
    downloader: &'a mut FakeDownloadService,
}

impl<'a> BrowserCapabilityStub<'a> {
    fn new(downloader: &'a mut FakeDownloadService) -> Self {
        Self { downloader }
    }

    fn invoke(&mut self, home: &Path) {
        let _ = assets::ensure_browser_asset(home, self.downloader).expect("ensure browser asset");
    }
}

#[test]
fn browser_asset_lazy_download() {
    let home = temp_home();
    let expected_browser_manifest = assets::browser_asset_manifest_path(&home);
    let built_binary = env!("CARGO_BIN_EXE_zstar");

    let startup_output = Command::new(built_binary)
        .env("HOME", &home)
        .output()
        .expect("run zstar startup");

    assert!(
        startup_output.status.success(),
        "startup should not depend on a browser asset: {}",
        String::from_utf8_lossy(&startup_output.stderr)
    );

    let mut downloader = FakeDownloadService::default();
    let mut browser = BrowserCapabilityStub::new(&mut downloader);

    browser.invoke(&home);
    browser.invoke(&home);

    assert_eq!(
        downloader.downloads, 1,
        "expected browser asset to download once and then reuse the cached manifest at {expected_browser_manifest:?}"
    );
}
