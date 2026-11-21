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

    #[test]
    fn stats_queries_read_only_visible_comics() {
        let source = include_str!("models/stats.rs");

        for (index, line) in source.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            for table in ["`Comic`", "`Occurrence`"] {
                assert!(
                    !line.contains(table),
                    "models/stats.rs:{} reads {table} directly; read `v_visible_comic` or \
                     `v_visible_occurrence` so hidden comics stay out of the public stats:\n{line}",
                    index + 1
                );
            }
        }
    }

    #[test]
    fn public_item_queries_read_only_visible_comics() {
        let occurrence: &[&str] = &["`Occurrence`"];
        let occurrence_and_comic: &[&str] = &["`Occurrence`", "`Comic`"];
        let functions = [
            (
                "item.rs",
                include_str!("models/item.rs"),
                "all_with_counts",
                occurrence,
            ),
            (
                "item.rs",
                include_str!("models/item.rs"),
                "first_and_last_apperance_and_count_by_id",
                occurrence,
            ),
            (
                "item.rs",
                include_str!("models/item.rs"),
                "occurrence_stats_by_id",
                occurrence_and_comic,
            ),
            (
                "item.rs",
                include_str!("models/item.rs"),
                "by_id_with_stats",
                occurrence_and_comic,
            ),
            (
                "item.rs",
                include_str!("models/item.rs"),
                "active_storylines_by_comic_id",
                occurrence,
            ),
            (
                "item.rs",
                include_str!("models/item.rs"),
                "related_items_by_id_and_type_with_mapping",
                occurrence,
            ),
            (
                "comic.rs",
                include_str!("models/comic.rs"),
                "all_with_item_id_mapped",
                occurrence,
            ),
            (
                "comic.rs",
                include_str!("models/comic.rs"),
                "random_comic_id_with_item_id",
                occurrence,
            ),
            (
                "occurrence.rs",
                include_str!("models/occurrence.rs"),
                "comic_id_occurrence_by_item_id",
                occurrence,
            ),
        ];

        for (file, source, function, tables) in functions {
            let body = function_source(source, function)
                .unwrap_or_else(|| panic!("models/{file} has no function `{function}`"));
            for line in body.lines() {
                if line.trim_start().starts_with("//") {
                    continue;
                }
                for table in tables {
                    assert!(
                        !line.contains(table),
                        "models/{file}: `{function}` reads {table} directly; read \
                         `v_visible_comic` or `v_visible_occurrence` so hidden comics stay out \
                         of the public item data:\n{line}"
                    );
                }
            }
        }
    }

    /// Returns the source from `pub async fn {name}` up to the next `pub async fn`.
    fn function_source<'s>(source: &'s str, name: &str) -> Option<&'s str> {
        const FN: &str = "pub async fn ";
        let start = source.find(&format!("{FN}{name}<"))?;
        let rest = &source[start + FN.len()..];
        Some(rest.find(FN).map_or(rest, |end| &rest[..end]))
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
