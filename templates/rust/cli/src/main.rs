use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Give a name to greet
    #[arg(short, long, default_value = "World")]
    name: String,
    /// Return health status
    #[arg(long, default_value_t = false)]
    health: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    if args.health {
        println!("{\"status\":\"ok\"}");
    } else {
        println!("Hello, {}!", args.name);
    }
    Ok(())
}
