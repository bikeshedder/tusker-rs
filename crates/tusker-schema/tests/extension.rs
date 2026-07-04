use tusker_schema::{
    diff::{ChangeType, DiffSql},
    models::{extension::Extension, schema::Schema},
};

#[test]
fn creates_extensions() {
    let from = Schema::new("public");
    let mut to = Schema::new("public");
    let _ = to.extensions.insert(
        "hstore".into(),
        Extension {
            schema: "public".into(),
            name: "hstore".into(),
            version: "1.8".into(),
        },
    );

    assert_eq!(
        from.diff_extensions(&to).sql(),
        vec![(
            ChangeType::CreateExtension,
            "CREATE EXTENSION IF NOT EXISTS \"hstore\" WITH SCHEMA \"public\" VERSION '1.8';\n"
                .into()
        )]
    );
}

#[test]
fn alters_extension_schema_and_version() {
    let mut from = Schema::new("public");
    let _ = from.extensions.insert(
        "postgis".into(),
        Extension {
            schema: "public".into(),
            name: "postgis".into(),
            version: "3.4.0".into(),
        },
    );
    let mut to = Schema::new("extensions");
    let _ = to.extensions.insert(
        "postgis".into(),
        Extension {
            schema: "extensions".into(),
            name: "postgis".into(),
            version: "3.5.0".into(),
        },
    );

    assert_eq!(
        from.diff_extensions(&to).sql(),
        vec![
            (
                ChangeType::AlterExtension,
                "ALTER EXTENSION \"postgis\" SET SCHEMA \"extensions\";\n".into(),
            ),
            (
                ChangeType::AlterExtension,
                "ALTER EXTENSION \"postgis\" UPDATE TO '3.5.0';\n".into(),
            ),
        ]
    );
}

#[test]
fn drops_extensions() {
    let mut from = Schema::new("public");
    let _ = from.extensions.insert(
        "hstore".into(),
        Extension {
            schema: "public".into(),
            name: "hstore".into(),
            version: "1.8".into(),
        },
    );
    let to = Schema::new("public");

    assert_eq!(
        from.diff_extensions(&to).sql(),
        vec![(
            ChangeType::DropExtension,
            "DROP EXTENSION IF EXISTS \"hstore\";\n".into()
        )]
    );
}
