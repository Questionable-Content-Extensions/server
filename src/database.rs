use std::ops::Deref;

use crate::util::Environment;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlSslMode};

pub mod models;

#[derive(Clone)]
pub struct DbPool(sqlx::Pool<sqlx::MySql>);

pub type DbPoolConnection = sqlx::pool::PoolConnection<sqlx::MySql>;

impl DbPool {
    // Set up database connection pool
    pub async fn create() -> Self {
        let database_options = Environment::database_url()
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
    type Target = sqlx::Pool<sqlx::MySql>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
