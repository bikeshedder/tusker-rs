use std::collections::HashMap;

use itertools::Itertools;

use crate::diff::{diff, ChangeType, Diff, DiffSql};

use super::{constraint::Constraint, table::Table, view::View};

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Schema {
    pub name: String,
    pub tables: HashMap<String, Table>,
    pub views: HashMap<String, View>,
    pub constraints: HashMap<(String, String), Constraint>,
}

impl Schema {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            ..Default::default()
        }
    }
    pub fn diff_tables<'a>(&'a self, other: &'a Self) -> Diff<'a, Table> {
        diff(self.tables.values(), other.tables.values(), |table| {
            &table.name
        })
    }
    pub fn diff_constraints<'a>(&'a self, other: &'a Self) -> Diff<'a, Constraint> {
        diff(self.constraints.values(), other.constraints.values(), |c| {
            (&c.table, &c.name)
        })
    }
}

impl DiffSql for Diff<'_, Schema> {
    fn sql(&self) -> Vec<(ChangeType, String)> {
        let mut v = Vec::new();
        if !self.a_only.is_empty() {
            todo!("Schema creation not supported, yet.")
        }
        for (a, b) in &self.a_and_b {
            v.extend(a.diff_tables(b).sql());
            v.extend(a.diff_constraints(b).sql());
        }
        if !self.b_only.is_empty() {
            println!("{:?}", self.b_only);
            todo!("Schema creation not supported, yet.")
        }
        v
    }
}

pub fn join_sql(v: Vec<(ChangeType, String)>) -> String {
    v.into_iter().sorted().map(|t| t.1).join("\n")
}
