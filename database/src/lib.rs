use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlSslMode};
use std::ops::Deref;

pub mod models;

type DatabaseDriver = sqlx::MySql;
type DatabaseQueryResult = sqlx::mysql::MySqlQueryResult;

pub type DbPoolConnection = sqlx::pool::PoolConnection<DatabaseDriver>;

#[derive(Clone)]
pub struct DbPool(sqlx::Pool<DatabaseDriver>);

impl DbPool {
    // Set up database connection pool
    pub async fn create(database_url: &'static str) -> Self {
        let database_options = database_url
            .parse::<MySqlConnectOptions>()
            .expect("failed to parse database URL")
            .ssl_mode(MySqlSslMode::Disabled);

        Self(
            MySqlPoolOptions::new()
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
