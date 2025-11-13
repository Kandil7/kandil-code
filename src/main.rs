//! Kandil Code - Intelligent Development Platform
//! 
//! The main entrypoint for the Kandil Code CLI application.

use anyhow::Result;

mod cli;
mod core;
mod utils;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let args = cli::Cli::parse();
    cli::run(args).await
}