use crate::{
    diff::{ChangeType, Diff, DiffSql},
    queries::{FunctionRow, RoutineKind},
    sql::quote_ident,
};

#[derive(Debug, Eq, PartialEq)]
pub struct Function {
    pub schema: String,
    pub name: String,
    pub kind: RoutineKind,
    pub identity_arguments: String,
    pub definition: String,
}

impl Function {
    fn create_sql(&self) -> String {
        if self.definition.ends_with('\n') {
            self.definition.clone()
        } else {
            format!("{}\n", self.definition)
        }
    }

    fn drop_sql(&self) -> String {
        format!(
            "DROP FUNCTION {}.{}({});\n",
            quote_ident(&self.schema),
            quote_ident(&self.name),
            self.identity_arguments,
        )
    }
}

impl From<FunctionRow> for Function {
    fn from(row: FunctionRow) -> Self {
        Self {
            schema: row.schema,
            name: row.name,
            kind: row.kind,
            identity_arguments: row.identity_arguments,
            definition: row.definition,
        }
    }
}

impl DiffSql for Diff<'_, Function> {
    fn sql(&self) -> Vec<(ChangeType, String)> {
        let mut v = Vec::new();
        for a in &self.a_only {
            v.push((ChangeType::DropFunction, a.drop_sql()));
        }
        for (a, b) in &self.a_and_b {
            if a != b {
                v.push((ChangeType::DropFunction, a.drop_sql()));
                v.push((ChangeType::CreateFunction, b.create_sql()));
            }
        }
        for b in &self.b_only {
            v.push((ChangeType::CreateFunction, b.create_sql()));
        }
        v
    }
}
