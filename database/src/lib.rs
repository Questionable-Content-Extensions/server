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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    #[test]
    fn latest_definition_of_every_view_is_sql_security_invoker() {
        let views = latest_view_definitions();

        assert!(
            views.contains_key("v_comics_missing_cast"),
            "expected to find the editor data views in the migrations, found {:?}",
            views.keys().collect::<Vec<_>>()
        );
        for (name, statement) in &views {
            assert!(
                statement
                    .to_ascii_uppercase()
                    .contains("SQL SECURITY INVOKER"),
                "the latest definition of view `{name}` is not SQL SECURITY INVOKER:\n{statement}"
            );
        }
    }

    /// Maps each view name to the last `CREATE ... VIEW` statement that defines it, in migration
    /// order.
    fn latest_view_definitions() -> BTreeMap<String, String> {
        let mut migrations: Vec<_> = sqlx::migrate!("./migrations").migrations.to_vec();
        migrations.sort_by_key(|m| m.version);

        let mut views = BTreeMap::new();
        for migration in &migrations {
            for statement in migration.sql.as_str().split(';') {
                if let Some(name) = created_view_name(statement) {
                    views.insert(name, statement.trim().to_owned());
                }
            }
        }
        views
    }

    fn created_view_name(statement: &str) -> Option<String> {
        let upper = statement.to_ascii_uppercase();
        let create = upper.find("CREATE ")?;
        let view = create + upper[create..].find(" VIEW `")? + " VIEW `".len();
        let length = statement[view..].find('`')?;
        Some(statement[view..view + length].to_owned())
    }
}
