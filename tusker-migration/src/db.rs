use std::error::Error as StdError;

use time::OffsetDateTime;
use tokio_postgres::{
    error::{DbError, ErrorPosition as PgErrorPosition},
    Error as PgError,
};

use crate::error::Error;
use crate::file::MigrationFile;

pub struct Database {
    pub client: tokio_postgres::Client,
}

impl Database {
    pub async fn connect(pg_config: &tokio_postgres::Config) -> Result<Database, Error> {
        let (client, connection) = pg_config
            .connect(tokio_postgres::NoTls)
            .await
            .map_err(|e| Error::Pg("Unable to connect to database".into(), e))?;
        tokio::spawn(connection);
        Ok(Database { client })
    }
    pub async fn migration_table_exists(&self) -> Result<bool, PgError> {
        let stmt = self
            .client
            .prepare("SELECT to_regclass('migration')::bigint")
            .await?;
        let result = self.client.query(&stmt, &[]).await?;
        Ok(match result.first() {
            Some(row) => row.get::<_, Option<i64>>(0).is_some(),
            None => false,
        })
    }
    pub async fn init(&self) -> Result<(), PgError> {
        let sql = include_str!("init.sql");
        self.client.simple_query(sql).await.map(|_| ())
    }
    pub async fn get_migrations(&self) -> Result<Vec<DbMigration>, PgError> {
        let stmt = self
            .client
            .prepare("SELECT number, name, hash FROM \"migration_current\"")
            .await?;
        let result = self.client.query(&stmt, &[]).await?;
        Ok(result
            .iter()
            .map(|row| DbMigration {
                number: row.get(0),
                name: row.get(1),
                hash: row.get(2),
            })
            .collect())
    }
    pub async fn get_migration_log(&self) -> Result<Vec<DbMigrationLog>, PgError> {
        let stmt = self
            .client
            .prepare("SELECT number, name, timestamp, operation::text FROM \"migration_log\"")
            .await?;
        let result = self.client.query(&stmt, &[]).await?;
        Ok(result
            .iter()
            .map(|row| DbMigrationLog {
                number: row.get(0),
                name: row.get(1),
                timestamp: row.get(2),
                operation: row.get(3),
            })
            .collect())
    }
    pub async fn update_migration(&self, migration_file: &MigrationFile) -> Result<(), PgError> {
        let sql = "SELECT migration_update($1, $2, $3)";
        self.client
            .execute(
                sql,
                &[
                    &migration_file.number,
                    &migration_file.name,
                    &migration_file.hash,
                ],
            )
            .await
            .map(|_| ())
    }
    pub async fn apply_migration(
        &self,
        migration_file: &MigrationFile,
        sql: &str,
    ) -> Result<(), PgError> {
        self.client.simple_query(sql).await.map(|_| ())?;
        // log that migration has been run
        let sql = "SELECT migration_insert($1, $2, $3)";
        self.client
            .execute(
                sql,
                &[
                    &migration_file.number,
                    &migration_file.name,
                    &migration_file.hash,
                ],
            )
            .await
            .map(|_| ())
    }
    pub async fn fake_migration(&self, migration_file: &MigrationFile) -> Result<(), PgError> {
        let sql = "SELECT migration_fake($1, $2, $3)";
        self.client
            .execute(
                sql,
                &[
                    &migration_file.number,
                    &migration_file.name,
                    &migration_file.hash,
                ],
            )
            .await
            .map(|_| ())
    }
    pub async fn remove_migration(&self, number: i32) -> Result<(), PgError> {
        let sql = "SELECT migration_delete($1)";
        self.client.execute(sql, &[&number]).await.map(|_| ())
    }
}

#[derive(Clone)]
pub struct DbMigration {
    pub number: i32,
    pub name: String,
    pub hash: Vec<u8>,
    //applied: std::time::
}

#[derive(Clone)]
pub struct DbMigrationLog {
    pub number: i32,
    pub name: String,
    pub timestamp: OffsetDateTime,
    pub operation: String,
}

pub fn to_sql_error(error: PgError, sql: &str) -> Error {
    // FIXME This function is really ugly and only works if the newline
    // is only '\n'.
    match error.source().unwrap().downcast_ref::<DbError>() {
        Some(db_error) => {
            println!("{}: {}", db_error.severity(), db_error.message());
            let position = match db_error.position() {
                Some(PgErrorPosition::Original(position)) => Some(*position),
                Some(PgErrorPosition::Internal { position, query: _ }) => Some(*position),
                None => None,
            };
            //let position = db_error.line();
            if let Some(position) = position {
                let position = position as usize;
                let line_begin = sql[..position].rfind('\n').map(|p| p + 1).unwrap_or(0);
                let line_end = sql[position..].find('\n').unwrap_or(sql.len()) + position;
                let line_position = position - line_begin;
                let mut remaining = position;
                let mut line_number = 1;
                for line in sql.lines() {
                    if remaining <= line.len() {
                        break;
                    } else {
                        remaining -= line.len() + 1; // FIXME assuming \n and no \r\n
                        line_number += 1;
                    }
                }
                let prefix = format!("LINE {}: ", line_number);
                let mut msg = format!("{}{}", prefix, &sql[line_begin..line_end]);
                // The position is 1 indexed. Thus 1 needs to be subtracted
                for _ in 0..(prefix.len() + line_position - 1) {
                    msg += " ";
                }
                msg += "^";
                Error::Sql(msg)
            } else {
                Error::Sql(format!("SQL error: {}", error))
            }
        }
        None => Error::Pg("Unknown error".into(), error),
    }
}
