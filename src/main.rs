use anyhow::Result;
use clap::ValueEnum;
use config::Config;

pub mod commands;
pub mod config;

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum Backend {
    Migrations,
    Schema,
    Database,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::new()?;
    commands::run(&cfg).await?;
    Ok(())
}
