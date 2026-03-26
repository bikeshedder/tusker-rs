use std::collections::HashMap;

use anyhow::Result;
use diff::{diff, Diff};
use models::{
    constraint::Constraint, r#enum::Enum, function::Function, schema::Schema, table::Table,
    sequence::Sequence, view::View,
};
use queries::Relkind;
use tokio_postgres::Client;

use crate::models::constraint::ConstraintType;

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
        // Enums
        let rows = tusker_query::query(
            client,
            queries::Enums {
                schema: schema.name.clone(),
            },
        )
        .await?;
        for row in rows {
            let e = Enum::from(row);
            schema.enums.insert(e.name.clone(), e);
        }
        // Sequences
        let rows = tusker_query::query(
            client,
            queries::Sequences {
                schema: schema.name.clone(),
            },
        )
        .await?;
        for row in rows {
            let sequence = Sequence::from(row);
            schema.sequences.insert(sequence.name.clone(), sequence);
        }
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
            let constraint = Constraint {
                schema: schema.name.clone(),
                table: row.table,
                name: row.name,
                r#type: row.r#type,
                definition: row.def,
            };
            if constraint.r#type == ConstraintType::NotNull {
                // Skip NOT NULL constraints introduced in PostgreSQL 18
                // It might be useful to support named not null constraints
                // in the future. That's why this is not filtered as part of
                // the query but here in the code.
                continue;
            }
            schema.constraints.insert(
                (constraint.table.clone(), constraint.name.clone()),
                constraint,
            );
        }
        // Functions
        let rows = tusker_query::query(
            client,
            queries::Functions {
                schema: schema.name.clone(),
            },
        )
        .await?;
        for row in rows {
            let function = Function::from(row);
            schema.functions.insert(
                (
                    function.name.clone(),
                    function.identity_arguments.clone(),
                ),
                function,
            );
        }
        schemas.insert(schema.name.clone(), schema);
    }

    Ok(Inspection { schemas })
}
