#![allow(missing_docs)]

use sqlx::migrate::MigrateError;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use std::ops::Deref;

pub mod models;

type DatabaseDriver = sqlx::MySql;
type DatabaseQueryResult = sqlx::mysql::MySqlQueryResult;

pub type DbPoolConnection = sqlx::pool::PoolConnection<DatabaseDriver>;
pub type DbTransaction<'c> = sqlx::Transaction<'c, sqlx::MySql>;

#[derive(Clone, Debug)]
pub struct DbPool(sqlx::Pool<DatabaseDriver>);

impl DbPool {
    /// # Panics
    ///
    /// Panics if the database URL cannot be parsed or the connection pool cannot be created.
    // Set up database connection pool
    pub async fn create(
        database_url: &'static str,
        max_connections: u32,
        min_connections: u32,
    ) -> Self {
        let database_options = database_url
            .parse::<MySqlConnectOptions>()
            .expect("failed to parse database URL");

        Self(
            MySqlPoolOptions::new()
                .max_connections(max_connections)
                .min_connections(min_connections)
                .connect_with(database_options)
                .await
                .expect("failed to create database pool"),
        )
    }
}

impl Deref for DbPool {
    type Target = sqlx::Pool<DatabaseDriver>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// # Errors
///
/// Returns a database error if the query fails.
#[tracing::instrument(skip(pool))]
pub async fn migrate(pool: &DbPool) -> Result<(), MigrateError> {
    sqlx::migrate!("./migrations").run(&**pool).await
}
