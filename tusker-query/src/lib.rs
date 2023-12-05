use deadpool_postgres::GenericClient;
use tokio_postgres::{types::ToSql, Row};

pub use tusker_query_derive::Query;

pub trait Query {
    const SQL: &'static str;
    type Row: FromRow;
    fn as_params(&self) -> Box<[&(dyn ToSql + Sync)]>;
}

pub trait FromRow {
    fn from_row(row: Row) -> Self;
}

pub use tusker_query_derive::FromRow;

pub struct Empty;

impl FromRow for Empty {
    fn from_row(_: Row) -> Self {
        Self
    }
}

pub async fn query_one<Q: Query>(
    client: &impl GenericClient,
    query: Q,
) -> Result<Q::Row, tokio_postgres::Error> {
    let stmt = client.prepare_cached(Q::SQL).await?;
    Ok(Q::Row::from_row(
        client.query_one(&stmt, &query.as_params()).await?,
    ))
}
