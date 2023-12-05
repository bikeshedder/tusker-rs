use std::{
    ffi::OsString,
    fs,
    num::{NonZeroI16, NonZeroU32},
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use sha2::{Digest, Sha512};
use tokio_postgres::Client;
use tusker_query_models::Column;

use crate::config::Config;

#[derive(Debug, Parser)]
pub struct QueryCommand {
    #[command(subcommand)]
    command: QuerySubcommand,
}
#[derive(Debug, Subcommand)]
pub enum QuerySubcommand {
    #[command(alias = "t")]
    Types(QueryTypeArgs),
}

#[derive(Debug, Parser)]
pub struct QueryTypeArgs {
    /// Filename containing the SQL query
    filename: OsString,
}

pub async fn cmd(cfg: &Config, args: QueryCommand) -> Result<()> {
    match args.command {
        QuerySubcommand::Types(args) => cmd_query_type(cfg, args).await?,
    }
    Ok(())
}

async fn cmd_query_type(cfg: &Config, args: QueryTypeArgs) -> Result<()> {
    let client = cfg.database.connect().await?;

    let content = fs::read(args.filename)?;
    let sql = String::from_utf8(content)?;

    let mut hasher = Sha512::new();
    hasher.update(&sql);
    let digest = hasher.finalize();

    let stmt = client.prepare(&sql).await?;

    let mut columns: Vec<Column> = Vec::new();
    for c in stmt.columns() {
        columns.push(Column {
            name: c.name().to_owned(),
            r#type: c.type_().to_string(),
            notnull: if let (Some(table_oid), Some(column_id)) = (c.table_oid(), c.column_id()) {
                Some(is_nullable(&client, table_oid, column_id).await?)
            } else {
                None
            },
        })
    }

    let query = tusker_query_models::Query {
        checksum: Vec::from_iter(digest),
        params: stmt.params().iter().map(|p| p.name().to_owned()).collect(),
        columns,
    };

    println!("{}", serde_json::to_string_pretty(&query)?);

    Ok(())
}

async fn is_nullable(client: &Client, table_id: NonZeroU32, column_id: NonZeroI16) -> Result<bool> {
    let stmt = client
        .prepare("SELECT attnotnull FROM pg_catalog.pg_attribute WHERE attrelid=$1 AND attnum=$2")
        .await?;
    let row = client
        .query_one(&stmt, &[&table_id.get(), &column_id.get()])
        .await?;
    Ok(row.get(0))
}
