use tokio_postgres::{types::ToSql, GenericClient, Row};

pub use tusker_query_derive::Query;

pub mod types;

pub trait Query: Sized {
    const SQL: &'static str;
    type Row: FromRow;
    fn as_params(&self) -> Box<[&(dyn ToSql + Sync)]>;
}

pub trait FromRow {
    fn from_row(row: Row) -> Self;
}

pub use tusker_query_derive::FromRow;

impl FromRow for () {
    fn from_row(_: Row) -> Self {}
}

pub async fn query_one<Q: Query>(
    client: &impl GenericClient,
    query: Q,
) -> Result<Q::Row, tokio_postgres::Error> {
    let stmt = client.prepare(Q::SQL).await?;
    Ok(Q::Row::from_row(
        client.query_one(&stmt, &query.as_params()).await?,
    ))
}

pub async fn query<Q: Query>(
    client: &impl GenericClient,
    query: Q,
) -> Result<Vec<Q::Row>, tokio_postgres::Error> {
    let stmt = client.prepare(Q::SQL).await?;
    let rows = client.query(&stmt, &query.as_params()).await?;
    Ok(rows.into_iter().map(Q::Row::from_row).collect())
}
