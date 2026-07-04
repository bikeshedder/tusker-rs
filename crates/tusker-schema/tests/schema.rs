use tusker_schema::{diff::ChangeType, models::schema::join_sql};

#[test]
fn join_sql_preserves_insertion_order_within_same_change_type() {
    let sql = join_sql(vec![
        (ChangeType::CreateRoutine, "second;\n".into()),
        (ChangeType::CreateRoutine, "first;\n".into()),
    ]);

    assert_eq!(sql, "second;\n\nfirst;\n");
}

#[test]
fn join_sql_creates_routines_before_tables() {
    let sql = join_sql(vec![
        (
            ChangeType::CreateTable,
            "CREATE TABLE uses_func (id integer);\n".into(),
        ),
        (
            ChangeType::CreateRoutine,
            "CREATE FUNCTION helper() RETURNS integer LANGUAGE sql AS $$ SELECT 1 $$;\n".into(),
        ),
    ]);

    assert_eq!(
        sql,
        "CREATE FUNCTION helper() RETURNS integer LANGUAGE sql AS $$ SELECT 1 $$;\n\nCREATE TABLE uses_func (id integer);\n"
    );
}
