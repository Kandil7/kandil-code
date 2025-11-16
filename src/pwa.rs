use anyhow::Result;
use std::fs;
use std::path::Path;

pub const INDEX_HTML: &str = include_str!("../pwa/index.html");
pub const MANIFEST: &str = include_str!("../pwa/manifest.webmanifest");
pub const SERVICE_WORKER: &str = include_str!("../pwa/service_worker.js");

pub fn write_assets(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir)?;
    fs::write(dir.join("index.html"), INDEX_HTML)?;
    fs::write(dir.join("manifest.webmanifest"), MANIFEST)?;
    fs::write(dir.join("sw.js"), SERVICE_WORKER)?;
    Ok(())
}
