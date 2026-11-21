//! Maximum lengths, in characters, of the text columns that API handlers write.

/// `Comic.title`
pub const COMIC_TITLE_MAX_CHARS: usize = 255;
/// `Comic.tagline`
pub const COMIC_TAGLINE_MAX_CHARS: usize = 255;
/// `Item.name`
pub const ITEM_NAME_MAX_CHARS: usize = 255;
/// `Item.short_name`
pub const ITEM_SHORT_NAME_MAX_CHARS: usize = 50;

#[cfg(test)]
mod db_tests {
    use super::*;
    use sqlx::MySqlPool;

    #[sqlx::test]
    async fn limits_match_the_schema(pool: MySqlPool) -> sqlx::Result<()> {
        for (table, column, expected) in [
            ("Comic", "title", COMIC_TITLE_MAX_CHARS),
            ("Comic", "tagline", COMIC_TAGLINE_MAX_CHARS),
            ("Item", "name", ITEM_NAME_MAX_CHARS),
            ("Item", "short_name", ITEM_SHORT_NAME_MAX_CHARS),
        ] {
            let actual: Option<u64> = sqlx::query_scalar(
                r"
                    SELECT `CHARACTER_MAXIMUM_LENGTH`
                    FROM `information_schema`.`COLUMNS`
                    WHERE `TABLE_SCHEMA` = DATABASE()
                        AND `TABLE_NAME` = ?
                        AND `COLUMN_NAME` = ?
                ",
            )
            .bind(table)
            .bind(column)
            .fetch_one(&pool)
            .await?;

            assert_eq!(
                actual,
                Some(expected as u64),
                "{table}.{column} length does not match its constant"
            );
        }
        Ok(())
    }
}
