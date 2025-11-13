use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Give a name to greet
    #[arg(short, long, default_value = "World")]
    name: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    println!("Hello, {}!", args.name);
    Ok(())
}