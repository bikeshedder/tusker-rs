//! Example application that embeds the `tusker-migration` runner with its SQL
//! migrations baked into the binary via [`rust-embed`](https://docs.rs/rust-embed).
//!
//! The `.sql` files under `db/migrations` are embedded at compile time and loaded
//! through [`RustEmbedSource`]. The rest wires `clap` into
//! [`tusker_migration::cli::cmd`] exactly like the `tusker` binary does, only with
//! an embedded source instead of a filesystem one — so there is no
//! `--migrations-dir` flag to configure.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p tusker-example-migration-rust-embed -- migration status
//! cargo run -p tusker-example-migration-rust-embed -- migration run
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links
)]
#![forbid(non_ascii_idents, unsafe_code)]
#![warn(
    deprecated_in_future,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]
#![allow(clippy::uninlined_format_args)]

use clap::{Parser, Subcommand};
use rust_embed::RustEmbed;
use tusker_migration::RustEmbedSource;

/// Migrations baked into the binary from `db/migrations`.
#[derive(RustEmbed)]
#[folder = "db/migrations"]
struct Migrations;

#[derive(Parser)]
#[command(about = "Example app embedding tusker-migration with rust-embed")]
struct Args {
    /// PostgreSQL connection string (libpq-style or URL).
    #[arg(long, default_value = "host=localhost user=postgres")]
    database_url: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Manage embedded database migrations (status, run, check, log, fix).
    Migration(tusker_migration::cli::Command),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let pg_config: tokio_postgres::Config = args.database_url.parse()?;
    let source = RustEmbedSource::<Migrations>::new();
    match args.command {
        Command::Migration(command) => {
            tusker_migration::cli::cmd(&pg_config, &source, &command).await?
        }
    }
    Ok(())
}
