use itertools::Itertools;

use crate::{
    diff::{ChangeType, Diff, DiffSql},
    queries::EnumRow,
    sql::quote_ident,
};

#[derive(Debug, Eq, PartialEq)]
pub struct Enum {
    pub schema: String,
    pub name: String,
    pub labels: Vec<String>,
}

impl Enum {
    fn create_sql(&self) -> String {
        format!(
            "CREATE TYPE {}.{} AS ENUM ({});\n",
            quote_ident(&self.schema),
            quote_ident(&self.name),
            self.labels.iter().map(|label| quote_literal(label)).join(", ")
        )
    }

    fn drop_sql(&self) -> String {
        format!(
            "DROP TYPE {}.{};\n",
            quote_ident(&self.schema),
            quote_ident(&self.name),
        )
    }

    fn alter_sql(&self, previous: &Self) -> Vec<(ChangeType, String)> {
        if self.labels.starts_with(&previous.labels) {
            self.labels[previous.labels.len()..]
                .iter()
                .map(|label| {
                    (
                        ChangeType::AlterType,
                        format!(
                            "ALTER TYPE {}.{} ADD VALUE {};\n",
                            quote_ident(&self.schema),
                            quote_ident(&self.name),
                            quote_literal(label),
                        ),
                    )
                })
                .collect()
        } else {
            vec![(
                ChangeType::AlterType,
                format!(
                    "-- WARNING: enum {}.{} changed incompatibly and no safe automatic migration was generated.\n\
-- Previous labels: {}\n\
-- Target labels: {}\n\
-- Suggested manual approach:\n\
-- 1. Change dependent columns to TEXT with an explicit USING cast.\n\
-- 2. Rewrite any rows, defaults, functions, or views that still reference removed/renamed values.\n\
-- 3. Recreate the enum type with the desired labels.\n\
-- 4. Cast dependent columns back to the enum type.\n\
DO $$\n\
BEGIN\n\
    RAISE EXCEPTION 'Unsafe enum migration required for {}.{}';\n\
END\n\
$$;\n",
                    quote_ident(&self.schema),
                    quote_ident(&self.name),
                    join_labels(&previous.labels),
                    join_labels(&self.labels),
                    self.schema,
                    self.name,
                ),
            )]
        }
    }
}

impl From<EnumRow> for Enum {
    fn from(row: EnumRow) -> Self {
        Self {
            schema: row.schema,
            name: row.name,
            labels: row.labels,
        }
    }
}

impl DiffSql for Diff<'_, Enum> {
    fn sql(&self) -> Vec<(ChangeType, String)> {
        let mut v = Vec::new();
        for a in &self.a_only {
            v.push((ChangeType::DropType, a.drop_sql()));
        }
        for (a, b) in &self.a_and_b {
            if a != b {
                v.extend(b.alter_sql(a));
            }
        }
        for b in &self.b_only {
            v.push((ChangeType::CreateType, b.create_sql()));
        }
        v
    }
}

fn quote_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn join_labels(labels: &[String]) -> String {
    labels.iter().map(|label| quote_literal(label)).join(", ")
}
