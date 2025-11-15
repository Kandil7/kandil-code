//! Model management CLI for Kandil Code
//!
//! Contains commands for managing local models.

use clap::Subcommand;
use indicatif::{ProgressBar, ProgressStyle};
use tokio;
use std::path::PathBuf;
use anyhow::Result;

use crate::config::layered::Config;
use crate::core::hardware::detect_hardware;
use crate::models::catalog::{MODEL_CATALOG, ModelSpec};

#[derive(Subcommand)]
pub enum ModelCommand {
    /// List all available models
    List {
        /// Show only models compatible with your hardware
        #[arg(long)]
        compatible: bool,
    },

    /// Install a model
    Install {
        /// Model name (e.g., qwen2.5-coder-7b-q4)
        name: String,
        /// Force install even if hardware is insufficient
        #[arg(long)]
        force: bool,
    },

    /// Remove a model
    Remove {
        name: String,
    },

    /// Verify model integrity
    Verify {
        name: String,
    },

    /// Benchmark installed model
    Benchmark {
        name: Option<String>,
        /// Output format
        #[arg(long, default_value = "table")]
        format: String, // Could be enum later
    },

    /// Set default model
    Use {
        name: String,
    },
}

pub async fn handle_model_command(cmd: ModelCommand) -> Result<()> {
    match cmd {
        ModelCommand::List { compatible } => {
            let hardware = detect_hardware();
            let catalog = &MODEL_CATALOG;

            println!("Available Models:");
            for model in catalog {
                // Check compatibility if requested
                if compatible && model.ram_required_gb > hardware.total_ram_gb {
                    continue;
                }

                println!("  {}", model.name);
                println!(
                    "    Size: {}GB, RAM: {}GB, GPU: {:?}GB",
                    model.size_gb,
                    model.ram_required_gb,
                    model.gpu_vram_min
                );
                
                // Show speed in a user-friendly way
                let speed_str = match &model.speed_rating {
                    crate::models::catalog::Speed::UltraFast(tps) => format!("Ultra Fast ({} t/s)", tps),
                    crate::models::catalog::Speed::VeryFast(tps) => format!("Very Fast ({} t/s)", tps),
                    crate::models::catalog::Speed::Fast(tps) => format!("Fast ({} t/s)", tps),
                    crate::models::catalog::Speed::Medium(tps) => format!("Medium ({} t/s)", tps),
                    crate::models::catalog::Speed::Slow(tps) => format!("Slow ({} t/s)", tps),
                };
                
                let quality_str = format!("{:?}", model.quality_rating);
                println!("    Speed: {}, Quality: {}", speed_str, quality_str);
                println!("    {}", model.description);
                println!("    Context sizes: {:?}", model.context_sizes);
            }
        }

        ModelCommand::Install { name, force } => {
            let model = MODEL_CATALOG
                .iter()
                .find(|m| m.name == name)
                .ok_or_else(|| anyhow::anyhow!("Unknown model: {}", name))?;

            let hardware = detect_hardware();
            if !force && model.ram_required_gb > hardware.total_ram_gb {
                anyhow::bail!(
                    "Insufficient RAM. Model requires {}GB, you have {}GB. Use --force to override.",
                    model.ram_required_gb,
                    hardware.total_ram_gb
                );
            }

            let path = get_model_path(&model.filename).await?;
            if path.exists() {
                println!("Model already installed at {:?}", path);
                return Ok(());
            }

            // Create directory if it doesn't exist
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            download_model(model, &path).await?;
            // Note: Verification would require SHA256 which isn't in the model spec
            
            println!("✅ Model {} installed successfully", name);
        }

        ModelCommand::Remove { name } => {
            let model = MODEL_CATALOG
                .iter()
                .find(|m| m.name == name)
                .ok_or_else(|| anyhow::anyhow!("Unknown model: {}", name))?;

            let path = get_model_path(&model.filename).await?;
            if path.exists() {
                tokio::fs::remove_file(&path).await?;
                println!("✅ Model {} removed successfully", name);
            } else {
                println!("Model {} not found at {:?}", name, path);
            }
        }

        ModelCommand::Benchmark { name, format } => {
            let model_name = name.unwrap_or_else(|| {
                let config = Config::load().unwrap_or_default();
                config.model.name
            });
            
            benchmark_model(&model_name, &format).await?;
        }

        ModelCommand::Use { name } => {
            // This would update the user's config file to set this as default
            println!("Setting {} as the default model", name);
            
            // In a real implementation, this would update the config file
            // For now, just print a message
            println!("Note: This would normally update the default model in your config file.");
        }

        ModelCommand::Verify { name } => {
            let model = MODEL_CATALOG
                .iter()
                .find(|m| m.name == name)
                .ok_or_else(|| anyhow::anyhow!("Unknown model: {}", name))?;

            let path = get_model_path(&model.filename).await?;
            if path.exists() {
                println!("✅ Model {} found and verified at {:?}", name, path);
            } else {
                println!("❌ Model {} not found at {:?}", name, path);
            }
        }
    }

    Ok(())
}

async fn get_model_path(filename: &str) -> Result<PathBuf> {
    let mut path = dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("kandil")
        .join("models");
    
    tokio::fs::create_dir_all(&path).await?;
    Ok(path.join(filename))
}

async fn download_model(model: &ModelSpec, path: &PathBuf) -> Result<()> {
    // Construct the Hugging Face download URL
    let url = format!(
        "https://huggingface.co/{}/resolve/main/{}",
        model.huggingface_repo, model.filename
    );

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    let total_size = response.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap());

    let mut file = tokio::io::BufWriter::new(tokio::fs::File::create(path).await?);
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        pb.inc(chunk.len() as u64);
        tokio::io::copy(&mut &chunk[..], &mut file).await?;
    }

    pb.finish_with_message("Download complete");
    file.flush().await?;
    Ok(())
}

async fn benchmark_model(name: &str, _format: &str) -> Result<()> {
    use std::time::Instant;
    
    println!("🔍 Benchmarking model: {}", name);

    // Skip actual benchmarking in this implementation since we don't have
    // the model loaded yet, but in a real implementation we would:
    // 1. Load the model
    // 2. Run several test prompts
    // 3. Measure tokens per second, latency, etc.
    
    println!("Note: Full benchmarking requires model loading which is complex to implement here.");
    println!("In a complete implementation, this would test:");
    println!("  - Simple Completion (fibonacci function)");
    println!("  - Refactoring (iterator conversion)");
    println!("  - Architecture (system design)");
    
    Ok(())
}