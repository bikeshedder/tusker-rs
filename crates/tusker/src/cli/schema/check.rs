use std::process::exit;

use anyhow::Result;
use clap::Parser;
use tusker_schema::{diff::DiffSql, models::schema::join_sql};

use crate::{config::Config, db::DiffDatabase};

use super::{diff::inspect_backend, Backend};

#[derive(Copy, Clone, Debug, Parser)]
pub(crate) struct CheckArgs {
    /// from-backend for the diff operation
    #[arg(default_value_t = Backend::Schema)]
    from: Backend,
    /// to-backend for the diff operation
    #[arg(default_value_t = Backend::Migrations)]
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

    let diff = from.diff(&to);
    let has_schema_add_drop = !diff.a_only.is_empty() || !diff.b_only.is_empty();
    let has_changes = if has_schema_add_drop {
        true
    } else {
        !join_sql(diff.sql()).trim().is_empty()
    };

    if !has_changes {
        println!("Schemas are identical");
        Ok(())
    } else {
        println!("Schemas differ: {} != {}", args.from, args.to);
        println!("Run `tusker diff` to see the differences");
        exit(1);
    }
}
