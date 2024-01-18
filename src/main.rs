use anyhow::Result;
use clap::ValueEnum;
use config::Config;

pub mod cli;
pub mod config;

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::new()?;
    cli::run(&cfg).await?;
    Ok(())
}
