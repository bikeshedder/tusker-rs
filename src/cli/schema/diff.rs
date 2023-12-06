use anyhow::Result;
use clap::Parser;

use crate::{config::Config, Backend};

#[derive(Debug, Parser)]
pub struct DiffArgs {
    /// from-backend for the diff operation
    #[arg(default_value_t = Backend::Schema)]
    from: Backend,
    /// to-backend for the diff operation
    #[arg(default_value_t = Backend::Migrations)]
    to: Backend,
    #[arg(long, short)]
    reverse: bool,
    /// throw an exception if drop-statements are generated
    #[arg(long, group = "group_safe")]
    safe: bool,
    /// don't throw an exception if drop-statements are generated
    #[arg(long, group = "group_safe")]
    r#unsafe: bool,
    /// output privilege differences (ie. grant/revoke statements)
    #[arg(long, group = "group_privileges")]
    with_privileges: bool,
    /// don't output privilege differences
    #[arg(long, group = "group_privileges")]
    without_privileges: bool,
}

pub async fn cmd(cfg: &Config, args: &DiffArgs) -> Result<()> {
    let client = cfg.database.connect().await?;
    let tables = tusker_schema::get_tables(&client).await?;
    // FIXME this is work in progress
    for table in &tables {
        println!("{}", table.create());
    }
    unimplemented!()
}
