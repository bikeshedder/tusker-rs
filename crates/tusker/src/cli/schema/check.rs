use std::process::exit;

use anyhow::Result;
use clap::Parser;
use tusker_schema::diff::DiffOptions;

use crate::{config::Config, db::DiffDatabase};

use super::{diff::inspect_backend, Backend};

#[derive(Copy, Clone, Debug, Parser)]
pub(crate) struct CheckArgs {
    // `check` reports "identical" exactly when `tusker diff` produces no
    // migration, so it defaults to the same direction as `diff`
    // (migrations -> schema) to evaluate the exact same change.
    /// from-backend for the diff operation
    #[arg(default_value_t = Backend::Migrations)]
    from: Backend,
    /// to-backend for the diff operation
    #[arg(default_value_t = Backend::Schema)]
    to: Backend,
    /// swaps the "from" and "to" arguments creating a reverse diff
    #[arg(long, short)]
    reverse: bool,
    /// check privilege differences (ie. grant/revoke statements)
    #[arg(long, group = "group_privileges")]
    with_privileges: bool,
    /// don't check privilege differences
    #[arg(long, group = "group_privileges")]
    without_privileges: bool,
}

pub(crate) async fn cmd(cfg: &Config, args: &CheckArgs) -> Result<()> {
    let mut db = DiffDatabase::new(&cfg.database).await?;
    db.create().await?;
    let from = inspect_backend(cfg, &mut db, args.from).await?;
    let to = inspect_backend(cfg, &mut db, args.to).await?;
    db.drop().await?;
    if from.equivalent(&to, &DiffOptions::default()) {
        println!("Schemas are identical");
        Ok(())
    } else {
        println!("Schemas differ: {} != {}", args.from, args.to);
        println!("Run `tusker diff` to see the differences");
        exit(1);
    }
}
