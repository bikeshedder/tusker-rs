use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use tokio_postgres::Client;

use crate::config::DatabaseConfig;

const TUSKER_COMMENT: &str = concat!(
    "CREATED BY TUSKER - If this table is left behind tusker probably ",
    "crashed and was not able to clean up after itself. Either try ",
    "running `tusker clean` or remove this database manually.",
);

pub struct DiffDatabase {
    client: Client,
    config: DatabaseConfig,
    pub dbname: String,
}

impl DiffDatabase {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let client = DatabaseConfig {
            dbname: "template1".into(),
            ..config.clone()
        }
        .connect()
        .await?;

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        Ok(Self {
            client,
            config: config.clone(),
            dbname: format!("{}_diff_{}", config.dbname, timestamp),
        })
    }
    pub async fn create(&self) -> Result<()> {
        self.client
            .simple_query(&format!("CREATE DATABASE {}", &self.dbname))
            .await?;
        self.client
            .simple_query(&format!(
                "COMMENT ON DATABASE {} IS '{}'",
                &self.dbname, TUSKER_COMMENT
            ))
            .await?;
        Ok(())
    }
    pub async fn connect(&self) -> Result<Client> {
        DatabaseConfig {
            dbname: self.dbname.clone(),
            ..self.config.clone()
        }
        .connect()
        .await
    }
    pub async fn drop(&self) -> Result<()> {
        self.client
            .execute(&format!("DROP DATABASE {}", &self.dbname), &[])
            .await?;
        Ok(())
    }
}
