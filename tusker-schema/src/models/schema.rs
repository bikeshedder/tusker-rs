use std::collections::HashMap;

use crate::diff::{diff, Diff};

use super::{table::Table, view::View};

#[derive(Debug, Default)]
pub struct Schema {
    pub name: String,
    pub tables: HashMap<String, Table>,
    pub views: HashMap<String, View>,
}

impl Schema {
    pub fn diff_tables<'a>(&'a self, other: &'a Self) -> Diff<'_, Table> {
        diff(self.tables.values(), other.tables.values(), |table| {
            &table.name
        })
    }
}
