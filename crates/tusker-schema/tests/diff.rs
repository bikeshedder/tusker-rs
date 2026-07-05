use std::env;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use rstest::rstest;
use tokio::fs;
use tokio::task::JoinHandle;
use tokio_postgres::{Client, Config, NoTls};
use tusker_schema::{
    diff::{DiffOptions, DiffSql},
    inspect,
    models::schema::join_sql,
    Inspection,
};

static NEXT_DB_ID: AtomicU64 = AtomicU64::new(0);

struct TestDatabase {
    admin_client: Client,
    db_client: Client,
    db_connection: JoinHandle<()>,
    dbname: String,
}

impl TestDatabase {
    async fn new() -> Result<Self> {
        let url = env::var("PG_URL").expect("Missing environment variable: PG_URL");
        let mut admin_config: Config = url.parse()?;
        admin_config.dbname("postgres");
        let (admin_client, admin_connection) = admin_config.connect(NoTls).await?;
        tokio::spawn(admin_connection);

        let unique_id = NEXT_DB_ID.fetch_add(1, Ordering::Relaxed);
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let dbname = format!("tusker_schema_test_{}_{}", timestamp, unique_id);
        admin_client
            .simple_query(&format!("CREATE DATABASE {}", dbname))
            .await?;

        let mut db_config: Config = url.parse()?;
        db_config.dbname(&dbname);
        let (db_client, db_connection_fut) = db_config.connect(NoTls).await?;
        let db_connection = tokio::spawn(async move {
            let _ = db_connection_fut.await;
        });

        Ok(Self {
            admin_client,
            db_client,
            db_connection,
            dbname,
        })
    }

    async fn cleanup(self) -> Result<()> {
        let TestDatabase {
            admin_client,
            db_client,
            db_connection,
            dbname,
        } = self;
        drop(db_client);
        let _ = db_connection.await;
        admin_client
            .simple_query(&format!("DROP DATABASE {} WITH (FORCE)", dbname))
            .await?;
        Ok(())
    }
}

async fn inspect_sql(client: &mut Client, sql: &str) -> Result<Inspection> {
    let txn = client.transaction().await.unwrap();
    txn.simple_query(sql).await?;
    let inspection = inspect(&txn.client()).await.unwrap();
    txn.rollback().await?;
    Ok(inspection)
}

async fn apply_and_inspect(
    client: &mut Client,
    base_sql: &str,
    diff_sql: &str,
) -> Result<Inspection> {
    let txn = client.transaction().await.unwrap();
    txn.simple_query(base_sql).await?;
    if !diff_sql.trim().is_empty() {
        txn.simple_query(diff_sql).await?;
    }
    let inspection = inspect(&txn.client()).await.unwrap();
    txn.rollback().await?;
    Ok(inspection)
}

fn is_runnable_migration(sql: &str) -> bool {
    // FIXME this way of detecting runnable migrations is not 100% foolproof.
    // If the schema contains a comment or some other part of the SQL with that
    // exact string it will return a false negative.
    !sql.contains("RAISE EXCEPTION 'Unsafe enum migration required")
        && !sql.contains("RAISE EXCEPTION 'Unsupported schema change for table")
}

/// Optional per-fixture configuration, read from a `test.toml` file in the
/// fixture directory. Every field is optional; a fixture without a `test.toml`
/// behaves as if all defaults were specified. A `README.md` in the same
/// directory (ignored here) may explain what the fixture is about.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
struct TestConfig {
    /// Minimum `server_version_num` required to run the fixture (e.g. `180000`
    /// for PostgreSQL 18). Fixtures that exercise version-specific features are
    /// skipped on older servers.
    min_server_version: Option<i32>,
    /// Diff options applied to the fixture, e.g. `[diff] removed_enum_value`.
    diff: DiffOptions,
}

async fn read_test_config(path: &Path) -> TestConfig {
    match fs::read_to_string(path.join("test.toml")).await {
        Ok(raw) => toml::from_str(&raw).expect("invalid test.toml"),
        Err(_) => TestConfig::default(),
    }
}

/// Reads `server_version_num` (e.g. `180004` for PostgreSQL 18.4).
async fn server_version_num(client: &Client) -> Result<i32> {
    let row = client.query_one("SHOW server_version_num", &[]).await?;
    let raw: String = row.get(0);
    Ok(raw.trim().parse()?)
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
    #[dirs]
    #[files("tests/*")]
    #[exclude("\\.rs$")]
    path: PathBuf,
) {
    let mut test_db = TestDatabase::new().await.unwrap();

    let config = read_test_config(&path).await;
    let opts = config.diff;

    // Fixtures that exercise version-specific features declare a
    // `min_server_version` in their `test.toml`; skip them on older servers.
    if let Some(min) = config.min_server_version {
        let version = server_version_num(&test_db.db_client).await.unwrap();
        if version < min {
            test_db.cleanup().await.unwrap();
            return;
        }
    }

    let client = &mut test_db.db_client;
    let a_sql = fs::read_to_string(path.join("a.sql")).await.unwrap();
    let b_sql = fs::read_to_string(path.join("b.sql")).await.unwrap();
    let up_sql = fs::read_to_string(path.join("up.sql")).await.unwrap();
    let down_sql = fs::read_to_string(path.join("down.sql")).await.unwrap();

    let a = inspect_sql(client, &a_sql).await.unwrap();
    let b = inspect_sql(client, &b_sql).await.unwrap();

    // test up migration
    let up_diff = a.diff(&b);
    let up_diff_sql = join_sql(up_diff.sql(&opts));
    assert_eq!(up_diff_sql, up_sql);

    let down_diff = b.diff(&a);
    let down_diff_sql = join_sql(down_diff.sql(&opts));
    assert_eq!(down_diff_sql, down_sql);

    let a_a_diff = a.diff(&a);
    assert!(a_a_diff.sql(&opts).is_empty());

    let b_b_diff = b.diff(&b);
    assert!(b_b_diff.sql(&opts).is_empty());

    // Applying the generated migration must reach a schema that `check`
    // considers equivalent to the target. It need not be structurally identical:
    // e.g. an ignored enum-value removal leaves the extra value in place, which
    // is equivalent under that option.
    if is_runnable_migration(&up_diff_sql) {
        let migrated_up = apply_and_inspect(client, &a_sql, &up_diff_sql)
            .await
            .unwrap();
        assert!(
            migrated_up.equivalent(&b, &opts),
            "up migration did not reach a schema equivalent to the target"
        );
    }

    if is_runnable_migration(&down_diff_sql) {
        let migrated_down = apply_and_inspect(client, &b_sql, &down_diff_sql)
            .await
            .unwrap();
        assert!(
            migrated_down.equivalent(&a, &opts),
            "down migration did not reach a schema equivalent to the target"
        );
    }

    test_db.cleanup().await.unwrap();
}
