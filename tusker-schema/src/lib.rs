use std::collections::HashMap;

use anyhow::Result;
use diff::{diff, Diff};
use models::{constraint::Constraint, schema::Schema, table::Table, view::View};
use queries::Relkind;
use tokio_postgres::Client;

pub mod diff;
pub mod models;
pub mod queries;
pub(crate) mod sql;

#[derive(Debug, Eq, PartialEq)]
pub struct Inspection {
    pub schemas: HashMap<String, Schema>,
}

impl Inspection {
    pub fn empty() -> Self {
        Self {
            schemas: Default::default(),
        }
    }
    pub fn diff<'a>(&'a self, other: &'a Self) -> Diff<'a, Schema> {
        diff(self.schemas.values(), other.schemas.values(), |schema| {
            &schema.name
        })
    }
}

pub async fn inspect(client: &Client) -> Result<Inspection> {
    let mut schemas: HashMap<String, Schema> = HashMap::new();
    let rows = tusker_query::query(client, queries::Schemas {}).await?;
    for schema in rows {
        let mut schema = Schema::new(&schema.name);
        // Tables
        let rows = tusker_query::query(
            client,
            queries::Classes {
                schema: schema.name.clone(),
            },
        )
        .await?;
        for cls in rows {
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
        // Constraints
        let rows = tusker_query::query(
            client,
            queries::Constraints {
                schema: schema.name.clone(),
            },
        )
        .await?;
        for row in rows {
            schema.constraints.insert(
                (row.table.clone(), row.name.clone()),
                Constraint {
                    schema: schema.name.clone(),
                    table: row.table,
                    name: row.name,
                    r#type: row.r#type,
                    definition: row.def,
                },
            );
        }
        schemas.insert(schema.name.clone(), schema);
    }

    Ok(Inspection { schemas })
}
