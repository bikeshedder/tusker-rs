use crate::{
    diff::{ChangeType, Diff, DiffSql},
    queries::ExtensionRow,
    sql::quote_ident,
};

#[derive(Debug, Clone, Eq, PartialEq)]
/// A PostgreSQL extension definition.
pub struct Extension {
    /// Schema where the extension is installed.
    pub schema: String,
    /// Extension name.
    pub name: String,
    /// Installed extension version.
    pub version: String,
}

impl Extension {
    fn create_sql(&self) -> String {
        format!(
            "CREATE EXTENSION IF NOT EXISTS {} WITH SCHEMA {} VERSION '{}';\n",
            quote_ident(&self.name),
            quote_ident(&self.schema),
            quote_literal(&self.version),
        )
    }

    fn drop_sql(&self) -> String {
        format!("DROP EXTENSION IF EXISTS {};\n", quote_ident(&self.name),)
    }

    fn alter_sql(&self, previous: &Self) -> Vec<(ChangeType, String)> {
        let mut statements = Vec::new();

        if self.schema != previous.schema {
            statements.push((
                ChangeType::AlterExtension,
                format!(
                    "ALTER EXTENSION {} SET SCHEMA {};\n",
                    quote_ident(&self.name),
                    quote_ident(&self.schema),
                ),
            ));
        }

        if self.version != previous.version {
            statements.push((
                ChangeType::AlterExtension,
                format!(
                    "ALTER EXTENSION {} UPDATE TO '{}';\n",
                    quote_ident(&self.name),
                    quote_literal(&self.version),
                ),
            ));
        }

        statements
    }
}

impl From<ExtensionRow> for Extension {
    fn from(row: ExtensionRow) -> Self {
        Self {
            schema: row.schema,
            name: row.name,
            version: row.version,
        }
    }
}

impl DiffSql for Diff<'_, Extension> {
    fn sql(&self) -> Vec<(ChangeType, String)> {
        let mut v = Vec::new();
        for a in &self.a_only {
            v.push((ChangeType::DropExtension, a.drop_sql()));
        }
        for (a, b) in &self.a_and_b {
            if a != b {
                v.extend(b.alter_sql(a));
            }
        }
        for b in &self.b_only {
            v.push((ChangeType::CreateExtension, b.create_sql()));
        }
        v
    }
}

fn quote_literal(value: &str) -> String {
    value.replace('\'', "''")
}
