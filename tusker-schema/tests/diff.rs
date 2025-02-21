use std::env;
use std::path::PathBuf;

use anyhow::Result;
use rstest::rstest;
use tokio::fs;
use tokio_postgres::{Client, NoTls};
use tusker_schema::{diff::DiffSql, inspect, models::schema::join_sql, Inspection};

async fn connect() -> Result<tokio_postgres::Client, tokio_postgres::Error> {
    let url = env::var("PG_URL").expect("Missing environment variable: PG_URL");
    let (client, connection) = tokio_postgres::connect(&url, NoTls).await?;
    tokio::spawn(connection);
    Ok(client)
}

async fn inspect_sql(client: &mut Client, sql: &str) -> Result<Inspection> {
    let txn = client.transaction().await.unwrap();
    txn.simple_query(sql).await?;
    let inspection = inspect(&txn.client()).await.unwrap();
    txn.rollback().await?;
    Ok(inspection)
}

/*
#[tokio::test]
async fn test_basic() {
    let mut client = connect().await.unwrap();
    let inspection = inspect_sql(&mut client, include_str!("sql/0001.b.sql"))
        .await
        .unwrap();
    let schema = inspection
        .schemas
        .get("public")
        .expect("No 'public' schema");
    let table = schema.tables.get("a").expect("Table 'a' missing");
    let table_expected = models::table::Table {
        name: "a".into(),
        schema: "public".into(),
        kind: Relkind::OrdinaryTable,
        columns: vec![
            models::column::Column {
                name: "id".into(),
                r#type: "bigint".into(),
                notnull: true,
                identity: models::column::Identity::Always,
                default: None,
                generated: models::column::Generated::No,
            },
            models::column::Column {
                name: "name".into(),
                r#type: "character varying(50)".into(),
                notnull: true,
                identity: models::column::Identity::No,
                default: None,
                generated: models::column::Generated::No,
            },
            models::column::Column {
                name: "age".into(),
                r#type: "integer".into(),
                notnull: false,
                identity: models::column::Identity::No,
                default: None,
                generated: models::column::Generated::No,
            },
        ],
    };
    assert_eq!(table, &table_expected);
    let constraint_expected = Constraint {
        schema: "public".into(),
        table: "a".into(),
        name: "a_pkey".into(),
        definition: "PRIMARY KEY (id)".into(),
    };
    let constraint = schema
        .constraints
        .get(&(table.name.clone(), "a_pkey".into()))
        .expect("Constraint 'a_pkey' missing");
    assert_eq!(constraint, &constraint_expected);
    assert_eq!(
        inspection.diff(&Inspection::empty()),
        tusker_schema::diff::Diff {
            a_and_b: vec![],
            a_only: vec![&Schema {
                name: "public".into(),
                tables: HashMap::from([("a".into(), table_expected)]),
                views: HashMap::new(),
                constraints: HashMap::from([(("a".into(), "a_pkey".into(),), constraint_expected)]),
            }],
            b_only: vec![],
        }
    );
}
     */

#[rstest]
#[tokio::test]
async fn diff(
    #[files("tests/*")]
    #[exclude("\\.rs$")]
    path: PathBuf,
) {
    let mut client = connect().await.unwrap();
    let a_sql = fs::read_to_string(path.join("a.sql")).await.unwrap();
    let b_sql = fs::read_to_string(path.join("b.sql")).await.unwrap();
    let up_sql = fs::read_to_string(path.join("up.sql")).await.unwrap();
    let down_sql = fs::read_to_string(path.join("down.sql")).await.unwrap();

    let a = inspect_sql(&mut client, &a_sql).await.unwrap();
    let b = inspect_sql(&mut client, &b_sql).await.unwrap();

    // test up migration
    let up_diff = a.diff(&b);
    let up_diff_sql = join_sql(up_diff.sql());
    assert_eq!(up_diff_sql, up_sql);

    let down_diff = b.diff(&a);
    let down_diff_sql = join_sql(down_diff.sql());
    assert_eq!(down_diff_sql, down_sql);

    let a_a_diff = a.diff(&a);
    assert!(a_a_diff.sql().is_empty());

    let b_b_diff = b.diff(&b);
    assert!(b_b_diff.sql().is_empty());

    // TODO Apply `a` and `diff_sql` and check if it does not differ to `b`
    // TODO Check reverse diff and check if it results in `a`
}
