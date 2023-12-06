use itertools::Itertools;
use postgres_types::FromSql;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug)]
pub struct Table {
    pub nspname: String,
    pub relname: String,
    pub relkind: Relkind,
    pub columns: Vec<Column>,
    pub viewdef: Option<String>,
}

fn quote_ident(ident: &str) -> String {
    // FIXME add escapes
    format!("\"{}\"", ident)
}

impl Table {
    pub fn create(&self) -> String {
        let cols = self.columns.iter().map(|col| col.sql()).join(",\n    ");
        format!(
            "CREATE TABLE {}.{} (\n    {}\n);",
            quote_ident(&self.nspname),
            quote_ident(&self.relname),
            cols
        )
    }
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct Column {
    pub num: u16,
    pub name: String,
    pub r#type: String,
    pub notnull: bool,
    pub identity: Identity,
    pub generated: Generated,
    pub default: Option<String>,
}

impl Column {
    pub fn sql(&self) -> String {
        // https://www.postgresql.org/docs/16/sql-createtable.html
        // FIXME
        let notnull = self.notnull.then_some(String::from("NOT NULL"));
        let generated = match self.generated {
            Generated::No => None,
            Generated::Stored => Some(format!(
                "GENERATED ALWAYS AS {} STORED",
                self.default.as_ref().unwrap()
            )),
        };
        let identity_opts = self
            .default
            .as_ref()
            .map(|d| format!("({})", d))
            .unwrap_or_default();
        let identity = match self.identity {
            Identity::No => None,
            Identity::Default => Some(format!("GENERATED BY DEFAULT AS IDENTITY{identity_opts}")),
            Identity::Always => Some(format!("GENERATED ALWAYS AS IDENTITY{identity_opts}")),
        };
        [
            Some(quote_ident(&self.name)),
            Some(self.r#type.to_owned()),
            notnull,
            generated,
            identity,
        ]
        .into_iter()
        .flatten()
        .join(" ")
    }
}

#[derive(Debug, Deserialize)]
pub enum Generated {
    #[serde(rename = "")]
    No,
    #[serde(rename = "s")]
    Stored,
}

#[derive(Debug, Deserialize)]
pub enum Identity {
    #[serde(rename = "")]
    No,
    #[serde(rename = "a")]
    Always,
    #[serde(rename = "d")]
    Default,
}