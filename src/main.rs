use anyhow::Result;
use clap::Parser;

mod config;

fn main() -> Result<()> {
    let _cli = config::Cli::parse();
    println!("amp-rs starting...");
    Ok(())
}
