use tusker_schema::models::column::{Column, Generated, Identity};

#[test]
fn generated_columns_use_parenthesized_expression_without_default_clause() {
    let column = Column {
        name: "vacation_during".into(),
        r#type: "daterange".into(),
        notnull: false,
        identity: Identity::No,
        generated: Generated::Stored,
        default: Some("daterange(start_date, end_date, '[]'::text)".into()),
    };

    assert_eq!(
        column.sql(),
        "\"vacation_during\" daterange GENERATED ALWAYS AS (daterange(start_date, end_date, '[]'::text)) STORED"
    );
}

#[test]
fn generated_columns_wrap_multiline_case_expressions() {
    let column = Column {
        name: "gross_minutes".into(),
        r#type: "integer".into(),
        notnull: false,
        identity: Identity::No,
        generated: Generated::Stored,
        default: Some(
            "\nCASE\n    WHEN ((start_time IS NULL) OR (end_time IS NULL)) THEN 0\n    ELSE (floor((EXTRACT(epoch FROM (end_time - start_time)) / (60)::numeric)))::integer\nEND"
                .into(),
        ),
    };

    assert_eq!(
        column.sql(),
        "\"gross_minutes\" integer GENERATED ALWAYS AS (\nCASE\n    WHEN ((start_time IS NULL) OR (end_time IS NULL)) THEN 0\n    ELSE (floor((EXTRACT(epoch FROM (end_time - start_time)) / (60)::numeric)))::integer\nEND) STORED"
    );
}
