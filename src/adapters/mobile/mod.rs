use crate::models::catalog::MODEL_CATALOG;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
struct MobileManifest {
    generated_at: DateTime<Utc>,
    platform: &'static str,
    models: Vec<ManifestModel>,
}

#[derive(Debug, Serialize)]
struct ManifestModel {
    name: String,
    filename: String,
    size_bytes: u64,
}

pub fn export_ios_bundle(models_dir: &Path) -> Result<PathBuf> {
    export_manifest_bundle(models_dir, "ios", "ios_export")
}

pub fn export_android_bundle(models_dir: &Path) -> Result<PathBuf> {
    export_manifest_bundle(models_dir, "android", "android_export")
}

fn export_manifest_bundle(
    models_dir: &Path,
    platform: &'static str,
    folder: &str,
) -> Result<PathBuf> {
    let export_dir = models_dir.join(folder);
    fs::create_dir_all(&export_dir)
        .with_context(|| format!("Unable to create {}", export_dir.display()))?;

    let manifest = MobileManifest {
        generated_at: Utc::now(),
        platform,
        models: collect_models(models_dir),
    };

    let manifest_path = export_dir.join("manifest.json");
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)
        .with_context(|| format!("Failed to write {}", manifest_path.display()))?;

    Ok(export_dir)
}

fn collect_models(models_dir: &Path) -> Vec<ManifestModel> {
    let mut entries = Vec::new();
    for spec in MODEL_CATALOG.iter() {
        let candidate = models_dir.join(&spec.filename);
        if let Ok(metadata) = fs::metadata(&candidate) {
            entries.push(ManifestModel {
                name: spec.name.to_string(),
                filename: spec.filename.to_string(),
                size_bytes: metadata.len(),
            });
        }
    }
    entries
}
