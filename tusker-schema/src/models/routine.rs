use crate::{
    diff::{ChangeType, Diff, DiffSql},
    queries::{RoutineKind, RoutineRow},
    sql::quote_ident,
};

#[derive(Debug, Eq, PartialEq)]
pub struct Routine {
    pub schema: String,
    pub name: String,
    pub kind: RoutineKind,
    pub identity_arguments: String,
    pub definition: String,
}

impl Routine {
    fn create_sql(&self) -> String {
        // `pg_get_functiondef` returns the body without a guaranteed trailing
        // statement terminator/newline, while our fixture output and migration
        // application logic expect a complete standalone statement.
        format!(
            "{};\n",
            self.definition.trim_end_matches('\n').trim_end_matches(';')
        )
    }

    fn drop_sql(&self) -> String {
        let thing = match self.kind {
            RoutineKind::Function => "FUNCTION",
            RoutineKind::Procedure => "PROCEDURE",
            RoutineKind::Aggregate => "AGGREGATE",
            RoutineKind::Window => "FUNCTION",
        };
        format!(
            "DROP {} {}.{}({});\n",
            thing,
            quote_ident(&self.schema),
            quote_ident(&self.name),
            self.identity_arguments,
        )
    }
}

impl From<RoutineRow> for Routine {
    fn from(row: RoutineRow) -> Self {
        Self {
            schema: row.schema,
            name: row.name,
            kind: row.kind,
            identity_arguments: row.identity_arguments,
            definition: row.definition,
        }
    }
}

impl DiffSql for Diff<'_, Routine> {
    fn sql(&self) -> Vec<(ChangeType, String)> {
        let mut v = Vec::new();
        for a in &self.a_only {
            v.push((ChangeType::DropRoutine, a.drop_sql()));
        }
        for (a, b) in &self.a_and_b {
            if a != b {
                v.push((ChangeType::DropRoutine, a.drop_sql()));
                v.push((ChangeType::CreateRoutine, b.create_sql()));
            }
        }
        for b in &self.b_only {
            v.push((ChangeType::CreateRoutine, b.create_sql()));
        }
        v
    }
}
