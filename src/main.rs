//! Kandil Code - Intelligent Development Platform
//!
//! The main entrypoint for the Kandil Code CLI application.

use anyhow::Result;
use clap::Parser;
use env_logger::Env;

mod adapters;
mod benchmark;
mod cache;
mod cli;
mod common;
mod config;
mod core;
mod enhanced_ui;
mod errors;
mod mobile;
mod models;
mod monitoring;
mod pwa;
mod security;
mod shutdown;
#[cfg(feature = "tui")]
mod tui;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().filter_or("RUST_LOG", "warn")).init();

    let args = cli::Cli::parse();
    cli::run(args).await
}
