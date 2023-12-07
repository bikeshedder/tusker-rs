use std::collections::HashMap;

use anyhow::Result;
use models::{schema::Schema, table::Table, view::View};
use queries::Relkind;
use tokio_postgres::Client;

pub(crate) mod diff;
pub mod models;
pub mod queries;
pub(crate) mod sql;

pub async fn inspect(client: &Client) -> Result<HashMap<String, Schema>> {
    let mut schemas: HashMap<String, Schema> = HashMap::new();
    let rows = tusker_query::query(client, queries::Classes {}).await?;
    for cls in rows {
        let schema = schemas.entry(cls.schema.clone()).or_default();
        match cls.relkind {
            Relkind::OrdinaryTable => {
                schema
                    .tables
                    .insert(cls.name.clone(), Table::try_from(cls)?);
            }
            Relkind::Index => {}
            Relkind::Sequence => {}
            Relkind::ToastTable => {}
            Relkind::View => {
                schema.views.insert(cls.name.clone(), View::try_from(cls)?);
            }
            Relkind::MaterializedView => {
                schema.views.insert(cls.name.clone(), View::try_from(cls)?);
            }
            Relkind::CompositeType => {}
            Relkind::ForeignTable => {}
            Relkind::PartitionedTable => {}
            Relkind::PartitionedIndex => {}
        };
    }
    Ok(schemas)
}
