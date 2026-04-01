use std::collections::HashMap;

use itertools::Itertools;

use crate::diff::{diff, ChangeType, Diff, DiffSql};

use super::{
    constraint::Constraint, domain::Domain, extension::Extension, r#enum::Enum, routine::Routine,
    sequence::Sequence, table::Table, trigger::Trigger, view::View,
};

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Schema {
    pub name: String,
    pub enums: HashMap<String, Enum>,
    pub domains: HashMap<String, Domain>,
    pub sequences: HashMap<String, Sequence>,
    pub extensions: HashMap<String, Extension>,
    pub tables: HashMap<String, Table>,
    pub views: HashMap<String, View>,
    pub routines: HashMap<(String, String), Routine>,
    pub triggers: HashMap<(String, String), Trigger>,
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
    pub fn diff_enums<'a>(&'a self, other: &'a Self) -> Diff<'a, Enum> {
        diff(self.enums.values(), other.enums.values(), |e| &e.name)
    }
    pub fn diff_domains<'a>(&'a self, other: &'a Self) -> Diff<'a, Domain> {
        diff(self.domains.values(), other.domains.values(), |d| &d.name)
    }
    pub fn diff_sequences<'a>(&'a self, other: &'a Self) -> Diff<'a, Sequence> {
        diff(self.sequences.values(), other.sequences.values(), |s| {
            &s.name
        })
    }
    pub fn diff_extensions<'a>(&'a self, other: &'a Self) -> Diff<'a, Extension> {
        diff(self.extensions.values(), other.extensions.values(), |e| {
            &e.name
        })
    }
    pub fn diff_constraints<'a>(&'a self, other: &'a Self) -> Diff<'a, Constraint> {
        diff(self.constraints.values(), other.constraints.values(), |c| {
            (&c.table, &c.name)
        })
    }
    pub fn diff_routines<'a>(&'a self, other: &'a Self) -> Diff<'a, Routine> {
        diff(self.routines.values(), other.routines.values(), |f| {
            (&f.name, &f.identity_arguments)
        })
    }
    pub fn diff_triggers<'a>(&'a self, other: &'a Self) -> Diff<'a, Trigger> {
        diff(self.triggers.values(), other.triggers.values(), |t| {
            (&t.table_name, &t.name)
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
            v.extend(a.diff_triggers(b).sql());
            v.extend(a.diff_enums(b).sql());
            v.extend(a.diff_domains(b).sql());
            v.extend(a.diff_sequences(b).sql());
            v.extend(a.diff_extensions(b).sql());
            v.extend(a.diff_routines(b).sql());
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
