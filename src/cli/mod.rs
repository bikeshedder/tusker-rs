use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::config::Config;

pub mod clean;
pub mod config;
pub mod query;
pub mod schema;

#[derive(Debug, Parser)]
#[command(name = "tusker")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Schema(schema::SchemaCommand),
    /// Remove all temporary databases, schemas and tables created by
    /// tusker
    Clean(clean::CleanArgs),
    /// Get types of a given query
    #[command(alias = "q")]
    Query(query::QueryCommand),
    /// Config
    #[command(alias = "cfg")]
    Config(config::ConfigCommand),
    /// Alias for "schema diff"
    #[command(alias = "d")]
    Diff(schema::diff::DiffArgs),
}

pub async fn run(cfg: &Config) -> Result<()> {
    let args = Cli::parse();
    match &args.command {
        Commands::Schema(cmd_args) => {
            schema::cmd(cfg, cmd_args).await?;
        }
        Commands::Clean(cmd_args) => {
            clean::cmd(cfg, cmd_args).await?;
        }
        Commands::Query(args) => query::cmd(cfg, args).await?,
        Commands::Config(args) => config::cmd(cfg, args).await?,
        Commands::Diff(args) => schema::diff::cmd(cfg, args).await?,
    }
    Ok(())
}
