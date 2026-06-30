use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use tokio::task::JoinHandle;
use tokio_postgres::{Client, Config, NoTls};
use tusker_schema::{diff::DiffSql, inspect, models::schema::join_sql, Inspection};

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
    let txn = client.transaction().await?;
    txn.simple_query(sql).await?;
    let inspection = inspect(&txn.client()).await?;
    txn.rollback().await?;
    Ok(inspection)
}

#[tokio::test]
async fn table_column_history_can_be_non_actionable_but_not_structurally_equal() {
    let mut test_db = TestDatabase::new().await.unwrap();
    let client = &mut test_db.db_client;

    let canonical_sql = r#"
        CREATE TABLE public.expense (
            id integer NOT NULL,
            name text NOT NULL,
            unit text NOT NULL,
            price integer NOT NULL,
            cost integer NOT NULL,
            creator_id text NOT NULL,
            updater_id text
        );
    "#;

    let historical_sql = r#"
        CREATE TABLE public.expense (
            id integer NOT NULL,
            name text NOT NULL,
            unit text NOT NULL,
            cost integer NOT NULL,
            creator_id text NOT NULL,
            updater_id text
        );

        ALTER TABLE public.expense
            DROP COLUMN cost,
            ADD COLUMN cost_cents integer NOT NULL;

        ALTER TABLE public.expense RENAME COLUMN cost_cents TO cost;
        ALTER TABLE public.expense RENAME COLUMN cost TO price;

        ALTER TABLE public.expense
            ADD COLUMN internal_cost integer NOT NULL DEFAULT 0;
        ALTER TABLE public.expense
            ALTER COLUMN internal_cost DROP DEFAULT;
        ALTER TABLE public.expense RENAME COLUMN internal_cost TO cost;
    "#;

    let canonical = inspect_sql(client, canonical_sql).await.unwrap();
    let historical = inspect_sql(client, historical_sql).await.unwrap();

    assert_ne!(
        canonical, historical,
        "inspection structs should differ due to column-order history"
    );

    let forward_sql = join_sql(canonical.diff(&historical).sql());
    assert!(
        forward_sql.trim().is_empty(),
        "expected no actionable diff SQL, got: {}",
        forward_sql
    );

    let reverse_sql = join_sql(historical.diff(&canonical).sql());
    assert!(
        reverse_sql.trim().is_empty(),
        "expected no actionable reverse diff SQL, got: {}",
        reverse_sql
    );

    test_db.cleanup().await.unwrap();
}
