use crate::util::Environment;
use sqlx::mysql::MySqlPoolOptions;

pub mod models;

pub type DbPool = sqlx::Pool<sqlx::MySql>;

// Set up database connection pool
pub async fn create_db_pool() -> DbPool {
    let database_url = Environment::database_url();

    MySqlPoolOptions::new()
        .connect(database_url)
        .await
        .expect("failed to create database pool")
}
