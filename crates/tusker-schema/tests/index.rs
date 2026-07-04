use tusker_schema::{
    diff::{ChangeType, Diff, DiffSql},
    models::index::Index,
};

fn index(name: &str, definition: &str) -> Index {
    Index {
        schema: "public".into(),
        table_name: "employees".into(),
        name: name.into(),
        definition: definition.into(),
    }
}

#[test]
fn creates_named_indexes() {
    let idx = index(
        "employees_tenant_id_idx",
        "CREATE INDEX employees_tenant_id_idx ON public.employees USING btree (tenant_id)",
    );
    let diff = Diff {
        a_only: vec![],
        a_and_b: vec![],
        b_only: vec![&idx],
    };

    assert_eq!(
        diff.sql(),
        vec![(
            ChangeType::CreateIndex,
            "CREATE INDEX employees_tenant_id_idx ON public.employees USING btree (tenant_id);\n"
                .into()
        )]
    );
}

#[test]
fn recreates_changed_unique_indexes() {
    let old = index(
        "employees_email_uidx",
        "CREATE UNIQUE INDEX employees_email_uidx ON public.employees USING btree (email)",
    );
    let new = index(
        "employees_email_uidx",
        "CREATE UNIQUE INDEX employees_email_uidx ON public.employees USING btree (lower(email))",
    );
    let diff = Diff {
        a_only: vec![],
        a_and_b: vec![(&old, &new)],
        b_only: vec![],
    };

    assert_eq!(
        diff.sql(),
        vec![
            (
                ChangeType::DropIndex,
                "DROP INDEX \"public\".\"employees_email_uidx\";\n".into(),
            ),
            (
                ChangeType::CreateIndex,
                "CREATE UNIQUE INDEX employees_email_uidx ON public.employees USING btree (lower(email));\n"
                    .into(),
            ),
        ]
    );
}
