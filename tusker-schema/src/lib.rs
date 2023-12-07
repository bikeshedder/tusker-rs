use schema::Table;
use tokio_postgres::{Client, Error};

pub mod queries;
pub mod schema;
pub(crate) mod sql;

pub async fn get_tables(client: &Client) -> Result<Vec<Table>, Error> {
    let rows = tusker_query::query(client, queries::Classes {}).await?;
    let tables = rows
        .into_iter()
        .map(|row| schema::Table {
            nspname: row.schema,
            relname: row.name,
            relkind: row.relkind,
            columns: row.columns.0,
            viewdef: row.viewdef,
        })
        .collect();
    Ok(tables)
}
