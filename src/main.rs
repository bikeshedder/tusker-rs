use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use commands::{
    check::CheckArgs, clean::CleanArgs, config::ConfigCommand, diff::DiffArgs, query::QueryCommand,
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
    /// Show differences between two schemas
    ///
    /// This command calculates the difference between two database schemas.
    /// The from- and to-parameter accept one of the following backends:
    /// migrations, schema, database
    #[command(alias = "d")]
    Diff(DiffArgs),
    /// Check for differences between schemas
    ///
    /// This command checks for differences between two or more schemas.
    /// Exit code 0 means that the schemas are all in sync. Otherwise
    /// the exit code 1 is used. This is useful for continuous integration
    /// checks.
    #[command(alias = "chk")]
    Check(CheckArgs),
    /// Remove all temporary databases, schemas and tables created by
    /// tusker
    Clean(CleanArgs),
    /// Get types of a given query
    #[command(alias = "q")]
    Query(QueryCommand),
    /// Config
    #[command(alias = "cfg")]
    Config(ConfigCommand),
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

    match args.command {
        Commands::Diff(cmd_args) => {
            unimplemented!("This command is not implemented, yet.");
        }
        Commands::Check(cmd_args) => {
            unimplemented!("This command is not implemented, yet.");
        }
        Commands::Clean(cmd_args) => {
            unimplemented!("This command is not implemented, yet.");
        }
        Commands::Query(args) => commands::query::cmd(&cfg, args).await?,
        Commands::Config(args) => commands::config::cmd(&cfg, args).await?,
    }
    Ok(())
}
