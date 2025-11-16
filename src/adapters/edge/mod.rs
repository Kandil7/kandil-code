use crate::models::catalog::MODEL_CATALOG;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
struct EdgeManifest {
    generated_at: DateTime<Utc>,
    target: &'static str,
    models: Vec<EdgeModelEntry>,
    notes: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct EdgeModelEntry {
    name: String,
    gguf_filename: String,
    recommended_onnx: String,
}

pub fn export_edge_snapshot(models_dir: &Path) -> Result<PathBuf> {
    let export_dir = models_dir.join("edge_export");
    fs::create_dir_all(&export_dir)
        .with_context(|| format!("Unable to create {}", export_dir.display()))?;

    let manifest = EdgeManifest {
        generated_at: Utc::now(),
        target: "edge",
        models: collect_edge_entries(models_dir),
        notes: vec![
            "Convert GGUF to ONNX using `kandil model quantize --format onnx_int8 <name>`.",
            "Transfer ONNX artifacts to your edge device under ~/.kandil/models/onnx.",
        ],
    };

    let manifest_path = export_dir.join("edge_manifest.json");
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)
        .with_context(|| format!("Failed to write {}", manifest_path.display()))?;

    Ok(export_dir)
}

fn collect_edge_entries(models_dir: &Path) -> Vec<EdgeModelEntry> {
    let mut entries = Vec::new();
    for spec in MODEL_CATALOG.iter() {
        let candidate = models_dir.join(&spec.filename);
        if candidate.exists() {
            let recommended = format!("{}_int8.onnx", spec.name);
            entries.push(EdgeModelEntry {
                name: spec.name.to_string(),
                gguf_filename: spec.filename.to_string(),
                recommended_onnx: recommended,
            });
        }
    }
    entries
}
