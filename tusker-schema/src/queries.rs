use tokio_postgres::types::Json;
use tusker_query::{FromRow, Query};

use crate::schema::{Column, Relkind};

#[derive(Query)]
#[query(sql = "classes.sql", row = Class)]
pub struct Classes {}

#[derive(FromRow)]
pub struct Class {
    pub schema: String,
    pub name: String,
    pub relkind: Relkind,
    pub columns: Json<Vec<Column>>,
    pub viewdef: Option<String>,
}
