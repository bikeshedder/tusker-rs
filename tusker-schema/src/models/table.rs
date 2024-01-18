use itertools::Itertools;
use thiserror::Error;

use crate::{
    diff::{diff, Diff},
    queries::{Class, Relkind},
    sql::quote_ident,
};

use super::column::Column;

#[derive(Debug)]
pub struct Table {
    pub schema: String,
    pub name: String,
    pub kind: Relkind,
    pub columns: Vec<Column>,
}

impl TryFrom<Class> for Table {
    type Error = InvalidRelkind;
    fn try_from(cls: Class) -> Result<Self, Self::Error> {
        Ok(Self {
            schema: cls.schema,
            name: cls.name,
            kind: cls.relkind,
            columns: cls.columns.0,
        })
    }
}

#[derive(Debug, Error)]
#[error("Unsupported table for table: {0}")]
pub struct InvalidRelkind(Relkind);

impl Table {
    pub fn create(&self) -> String {
        let cols = self.columns.iter().map(|col| col.sql()).join(",\n    ");
        format!(
            "CREATE TABLE {}.{} (\n    {}\n);",
            quote_ident(&self.schema),
            quote_ident(&self.name),
            cols
        )
    }
    pub fn diff_columns<'a>(&'a self, other: &'a Self) -> Diff<'a, Column> {
        diff(self.columns.iter(), other.columns.iter(), |c| &c.name)
    }
}
