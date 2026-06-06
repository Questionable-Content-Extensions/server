#[derive(Debug, sqlx::FromRow)]
pub struct ItemStats {
    pub id: u16,
    pub name: String,
    pub first_comic: Option<u16>,
    pub last_comic: Option<u16>,
    pub appearances: i64,
}

impl ItemStats {
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn cast<'e, 'c: 'e, E>(executor: E) -> sqlx::Result<Vec<Self>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            Self,
            r#"
                SELECT
                    `i`.`id`,
                    `i`.`name`,
                    MIN(`o`.`comic_id`) AS `first_comic`,
                    MAX(`o`.`comic_id`) AS `last_comic`,
                    COUNT(*) AS `appearances`
                FROM `Item` `i`
                JOIN `Occurrence` `o` ON `i`.`id` = `o`.`item_id`
                WHERE `i`.`type` = 'cast'
                GROUP BY `i`.`id`
                ORDER BY `appearances` DESC
            "#,
        )
        .fetch_all(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn locations<'e, 'c: 'e, E>(executor: E) -> sqlx::Result<Vec<Self>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            Self,
            r#"
                SELECT
                    `i`.`id`,
                    `i`.`name`,
                    MIN(`o`.`comic_id`) AS `first_comic`,
                    MAX(`o`.`comic_id`) AS `last_comic`,
                    COUNT(*) AS `appearances`
                FROM `Item` `i`
                JOIN `Occurrence` `o` ON `i`.`id` = `o`.`item_id`
                WHERE `i`.`type` = 'location'
                GROUP BY `i`.`id`
                ORDER BY `appearances` DESC
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct CoAppearance {
    pub character1_id: u16,
    pub character1_name: String,
    pub character2_id: u16,
    pub character2_name: String,
    pub comics_together: i64,
}

impl CoAppearance {
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn top<'e, 'c: 'e, E>(executor: E) -> sqlx::Result<Vec<Self>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            Self,
            r#"
                SELECT
                    `i1`.`id` AS `character1_id`,
                    `i1`.`name` AS `character1_name`,
                    `i2`.`id` AS `character2_id`,
                    `i2`.`name` AS `character2_name`,
                    COUNT(*) AS `comics_together`
                FROM `Occurrence` `o1`
                JOIN `Occurrence` `o2`
                    ON `o1`.`comic_id` = `o2`.`comic_id`
                    AND `o1`.`item_id` < `o2`.`item_id`
                JOIN `Item` `i1` ON `o1`.`item_id` = `i1`.`id`
                JOIN `Item` `i2` ON `o2`.`item_id` = `i2`.`id`
                WHERE `i1`.`type` = 'cast' AND `i2`.`type` = 'cast'
                GROUP BY `o1`.`item_id`, `o2`.`item_id`
                ORDER BY `comics_together` DESC
                LIMIT 100
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct YearlyAppearanceRow {
    pub year: Option<i32>,
    pub id: u16,
    pub name: String,
    pub color_red: u8,
    pub color_green: u8,
    pub color_blue: u8,
    pub appearances: i64,
}

impl YearlyAppearanceRow {
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn all<'e, 'c: 'e, E>(executor: E) -> sqlx::Result<Vec<Self>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            Self,
            r#"
                SELECT
                    YEAR(`c`.`publish_date`) AS `year`,
                    `i`.`id`,
                    `i`.`name`,
                    `i`.`color_red`,
                    `i`.`color_green`,
                    `i`.`color_blue`,
                    COUNT(*) AS `appearances`
                FROM `Comic` `c`
                JOIN `Occurrence` `o` ON `c`.`id` = `o`.`comic_id`
                JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
                WHERE `c`.`publish_date` IS NOT NULL AND `i`.`type` = 'cast'
                GROUP BY `year`, `i`.`id`
                ORDER BY `year`, `appearances` DESC
            "#,
        )
        .fetch_all(executor)
        .await
    }
}
