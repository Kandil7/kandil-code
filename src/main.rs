//! Kandil Code - Intelligent Development Platform
//! 
//! The main entrypoint for the Kandil Code CLI application.

use anyhow::Result;
use clap::Parser;
use env_logger::Env;

mod cli;
mod core;
mod utils;
#[cfg(feature = "tui")]
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().filter_or("RUST_LOG", "warn")).init();
    
    let args = cli::Cli::parse();
    cli::run(args).await
}
