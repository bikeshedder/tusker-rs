use std::process::exit;

use anyhow::Result;
use clap::Parser;
use tusker_schema::{diff::DiffSql, models::schema::join_sql, Inspection};

use crate::{config::Config, db::DiffDatabase};

use super::{diff::inspect_backend, Backend};

fn has_actionable_changes(from: &Inspection, to: &Inspection) -> bool {
    let diff = from.diff(to);
    let has_schema_add_drop = !diff.a_only.is_empty() || !diff.b_only.is_empty();
    if has_schema_add_drop {
        true
    } else {
        !join_sql(diff.sql()).trim().is_empty()
    }
}

#[derive(Copy, Clone, Debug, Parser)]
pub(crate) struct CheckArgs {
    /// from-backend for the diff operation
    #[arg(default_value_t = Backend::Schema)]
    from: Backend,
    /// to-backend for the diff operation
    #[arg(default_value_t = Backend::Migrations)]
    to: Backend,
    /// swaps the "from" and "to" arguments creating a reverse diff
    #[arg(long, short)]
    reverse: bool,
    /// check privilege differences (ie. grant/revoke statements)
    #[arg(long, group = "group_privileges")]
    with_privileges: bool,
    /// don't check privilege differences
    #[arg(long, group = "group_privileges")]
    without_privileges: bool,
}

pub(crate) async fn cmd(cfg: &Config, args: &CheckArgs) -> Result<()> {
    let mut db = DiffDatabase::new(&cfg.database).await?;
    db.create().await?;
    let from = inspect_backend(cfg, &mut db, args.from).await?;
    let to = inspect_backend(cfg, &mut db, args.to).await?;
    db.drop().await?;

    let has_changes = has_actionable_changes(&from, &to);

    if !has_changes {
        println!("Schemas are identical");
        Ok(())
    } else {
        println!("Schemas differ: {} != {}", args.from, args.to);
        println!("Run `tusker diff` to see the differences");
        exit(1);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tusker_schema::{
        Inspection,
        models::{r#enum::Enum, schema::Schema},
    };

    use super::has_actionable_changes;

    fn inspection_with_schema(schema: Schema) -> Inspection {
        let mut schemas = HashMap::new();
        let _ = schemas.insert(schema.name.clone(), schema);
        Inspection { schemas }
    }

    #[test]
    fn reports_no_changes_when_diff_output_is_empty() {
        let from = inspection_with_schema(Schema::new("public"));
        let to = inspection_with_schema(Schema::new("public"));

        assert!(!has_actionable_changes(&from, &to));
    }

    #[test]
    fn reports_changes_for_schema_add_drop_guard() {
        let from = inspection_with_schema(Schema::new("public"));
        let to = inspection_with_schema(Schema::new("other"));

        assert!(has_actionable_changes(&from, &to));
    }

    #[test]
    fn reports_changes_when_actionable_sql_exists() {
        let mut from_schema = Schema::new("public");
        let _ = from_schema.enums.insert(
            "ticket_state".into(),
            Enum {
                schema: "public".into(),
                name: "ticket_state".into(),
                labels: vec!["new".into()],
            },
        );

        let mut to_schema = Schema::new("public");
        let _ = to_schema.enums.insert(
            "ticket_state".into(),
            Enum {
                schema: "public".into(),
                name: "ticket_state".into(),
                labels: vec!["new".into(), "assigned".into()],
            },
        );

        let from = inspection_with_schema(from_schema);
        let to = inspection_with_schema(to_schema);

        assert!(has_actionable_changes(&from, &to));
    }
}
