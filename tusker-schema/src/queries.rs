use std::fmt;

use postgres_types::FromSql;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio_postgres::types::Json;
use tusker_query::{FromRow, Query};

use crate::models::{column::Column, constraint::ConstraintType};

#[derive(Query)]
#[query(sql="schemas", row=Schema)]
pub struct Schemas {}

#[derive(Debug, FromRow)]
pub struct Schema {
    pub name: String,
}

#[derive(Query)]
#[query(sql="classes", row=Class)]
pub struct Classes {
    pub schema: String,
}

#[derive(Debug, FromRow)]
pub struct Class {
    pub schema: String,
    pub name: String,
    pub relkind: Relkind,
    pub columns: Json<Vec<Column>>,
    pub viewdef: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Relkind {
    #[serde(rename = "r")]
    OrdinaryTable,
    #[serde(rename = "i")]
    Index,
    #[serde(rename = "S")]
    Sequence,
    #[serde(rename = "t")]
    ToastTable,
    #[serde(rename = "v")]
    View,
    #[serde(rename = "m")]
    MaterializedView,
    #[serde(rename = "c")]
    CompositeType,
    #[serde(rename = "f")]
    ForeignTable,
    #[serde(rename = "p")]
    PartitionedTable,
    #[serde(rename = "I")]
    PartitionedIndex,
}

impl fmt::Display for Relkind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromSql<'_> for Relkind {
    fn from_sql(
        _ty: &postgres_types::Type,
        raw: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(match raw {
            b"r" => Self::OrdinaryTable,
            b"i" => Self::Index,
            b"S" => Self::Sequence,
            b"t" => Self::ToastTable,
            b"v" => Self::View,
            b"m" => Self::MaterializedView,
            b"c" => Self::CompositeType,
            b"p" => Self::PartitionedTable,
            b"I" => Self::PartitionedIndex,
            x => Err(UnsupportedRelkind(x.to_owned()))?,
        })
    }
    fn accepts(ty: &postgres_types::Type) -> bool {
        *ty == postgres_types::Type::CHAR
    }
}

#[derive(Error, Debug)]
#[error("Unsupported relkind value")]
struct UnsupportedRelkind(Vec<u8>);

#[derive(Query)]
#[query(sql="constraints", row=Constraint)]
pub struct Constraints {
    pub schema: String,
}

#[derive(Debug, FromRow)]
pub struct Constraint {
    pub table: String,
    pub name: String,
    pub r#type: ConstraintType,
    pub def: String,
}
