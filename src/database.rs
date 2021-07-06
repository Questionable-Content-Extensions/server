use std::ops::Deref;

use crate::util::Environment;
use sqlx::mysql::MySqlPoolOptions;

pub mod models;

#[derive(Clone)]
pub struct DbPool(sqlx::Pool<sqlx::MySql>);

impl DbPool {
    // Set up database connection pool
    pub async fn create() -> Self {
        let database_url = Environment::database_url();

        Self(
            MySqlPoolOptions::new()
                .connect(database_url)
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
