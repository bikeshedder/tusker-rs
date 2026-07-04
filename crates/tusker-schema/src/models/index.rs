use crate::{
    diff::{ChangeType, Diff, DiffSql},
    queries::IndexRow,
    sql::quote_ident,
};

#[derive(Debug, Clone, Eq, PartialEq)]
/// A standalone PostgreSQL index definition.
pub struct Index {
    /// Schema that owns the index.
    pub schema: String,
    /// Table referenced by the index.
    pub table_name: String,
    /// Index name.
    pub name: String,
    /// Raw PostgreSQL `CREATE INDEX` statement.
    pub definition: String,
}

impl Index {
    fn create_sql(&self) -> String {
        format!(
            "{};\n",
            self.definition.trim_end_matches('\n').trim_end_matches(';')
        )
    }

    fn drop_sql(&self) -> String {
        format!(
            "DROP INDEX {}.{};\n",
            quote_ident(&self.schema),
            quote_ident(&self.name),
        )
    }
}

impl From<IndexRow> for Index {
    fn from(row: IndexRow) -> Self {
        Self {
            schema: row.schema,
            table_name: row.table_name,
            name: row.name,
            definition: row.definition,
        }
    }
}

impl DiffSql for Diff<'_, Index> {
    fn sql(&self) -> Vec<(ChangeType, String)> {
        let mut v = Vec::new();
        for a in &self.a_only {
            v.push((ChangeType::DropIndex, a.drop_sql()));
        }
        for (a, b) in &self.a_and_b {
            if a != b {
                v.push((ChangeType::DropIndex, a.drop_sql()));
                v.push((ChangeType::CreateIndex, b.create_sql()));
            }
        }
        for b in &self.b_only {
            v.push((ChangeType::CreateIndex, b.create_sql()));
        }
        v
    }
}
