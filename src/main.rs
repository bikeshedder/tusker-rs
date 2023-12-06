use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use commands::{
    clean::CleanArgs,
    config::ConfigCommand,
    query::QueryCommand,
    schema::{diff::DiffArgs, SchemaCommand},
};
use config::Config;

pub mod commands;
pub mod config;

#[derive(Debug, Parser)]
#[command(name = "tusker")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Schema(SchemaCommand),
    /// Remove all temporary databases, schemas and tables created by
    /// tusker
    Clean(CleanArgs),
    /// Get types of a given query
    #[command(alias = "q")]
    Query(QueryCommand),
    /// Config
    #[command(alias = "cfg")]
    Config(ConfigCommand),
    /// Alias for "schema diff"
    #[command(alias = "d")]
    Diff(DiffArgs),
}

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
    let args = Cli::parse();
    let cfg = Config::new()?;

    match &args.command {
        Commands::Schema(cmd_args) => {
            commands::schema::cmd(&cfg, cmd_args).await?;
        }
        Commands::Clean(cmd_args) => {
            commands::clean::cmd(&cfg, cmd_args).await?;
        }
        Commands::Query(args) => commands::query::cmd(&cfg, args).await?,
        Commands::Config(args) => commands::config::cmd(&cfg, args).await?,
        Commands::Diff(args) => commands::schema::diff::cmd(&cfg, args).await?,
    }
    Ok(())
}
