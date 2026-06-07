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
    pub character1_appearances: i64,
    pub character2_id: u16,
    pub character2_name: String,
    pub character2_appearances: i64,
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
                    `app1`.`total` AS `character1_appearances`,
                    `i2`.`id` AS `character2_id`,
                    `i2`.`name` AS `character2_name`,
                    `app2`.`total` AS `character2_appearances`,
                    COUNT(*) AS `comics_together`
                FROM `Occurrence` `o1`
                JOIN `Occurrence` `o2`
                    ON `o1`.`comic_id` = `o2`.`comic_id`
                    AND `o1`.`item_id` < `o2`.`item_id`
                JOIN `Item` `i1` ON `o1`.`item_id` = `i1`.`id`
                JOIN `Item` `i2` ON `o2`.`item_id` = `i2`.`id`
                JOIN (
                    SELECT `item_id`, COUNT(*) AS `total`
                    FROM `Occurrence`
                    GROUP BY `item_id`
                ) `app1` ON `o1`.`item_id` = `app1`.`item_id`
                JOIN (
                    SELECT `item_id`, COUNT(*) AS `total`
                    FROM `Occurrence`
                    GROUP BY `item_id`
                ) `app2` ON `o2`.`item_id` = `app2`.`item_id`
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

#[derive(Copy, Clone, Debug, sqlx::FromRow)]
pub struct DebutsPerYearRow {
    pub year: Option<i32>,
    pub cast_debuts: i64,
    pub location_debuts: i64,
}

impl DebutsPerYearRow {
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
                    `first_app`.`year`,
                    COUNT(CASE WHEN `i`.`type` = 'cast' THEN 1 END) AS `cast_debuts`,
                    COUNT(CASE WHEN `i`.`type` = 'location' THEN 1 END) AS `location_debuts`
                FROM (
                    SELECT `o`.`item_id`, YEAR(MIN(`c`.`publish_date`)) AS `year`
                    FROM `Occurrence` `o`
                    JOIN `Comic` `c` ON `o`.`comic_id` = `c`.`id`
                    JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
                    WHERE `c`.`publish_date` IS NOT NULL
                      AND `i`.`type` IN ('cast', 'location')
                    GROUP BY `o`.`item_id`
                ) `first_app`
                JOIN `Item` `i` ON `first_app`.`item_id` = `i`.`id`
                GROUP BY `first_app`.`year`
                ORDER BY `first_app`.`year`
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Copy, Clone, Debug, sqlx::FromRow)]
pub struct YearlyOverviewRow {
    pub year: Option<i32>,
    pub total_cast: i64,
    pub new_cast: i64,
}

impl YearlyOverviewRow {
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
                    `yc`.`year`,
                    COUNT(*) AS `total_cast`,
                    COUNT(CASE WHEN `fd`.`debut_year` = `yc`.`year` THEN 1 END) AS `new_cast`
                FROM (
                    SELECT YEAR(`c`.`publish_date`) AS `year`, `o`.`item_id`
                    FROM `Occurrence` `o`
                    JOIN `Comic` `c` ON `o`.`comic_id` = `c`.`id`
                    JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
                    WHERE `c`.`publish_date` IS NOT NULL AND `i`.`type` = 'cast'
                    GROUP BY YEAR(`c`.`publish_date`), `o`.`item_id`
                ) `yc`
                JOIN (
                    SELECT `o`.`item_id`, YEAR(MIN(`c`.`publish_date`)) AS `debut_year`
                    FROM `Occurrence` `o`
                    JOIN `Comic` `c` ON `o`.`comic_id` = `c`.`id`
                    JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
                    WHERE `c`.`publish_date` IS NOT NULL AND `i`.`type` = 'cast'
                    GROUP BY `o`.`item_id`
                ) `fd` ON `yc`.`item_id` = `fd`.`item_id`
                GROUP BY `yc`.`year`
                ORDER BY `yc`.`year`
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Copy, Clone, Debug, sqlx::FromRow)]
pub struct PublicationMonthRow {
    pub month: Option<i32>,
    pub comics: i64,
}

impl PublicationMonthRow {
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
                SELECT MONTH(`publish_date`) AS `month`, COUNT(*) AS `comics`
                FROM `Comic`
                WHERE `publish_date` IS NOT NULL
                GROUP BY `month`
                ORDER BY `month`
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Copy, Clone, Debug, sqlx::FromRow)]
pub struct PublicationDowRow {
    pub dow: Option<i32>,
    pub comics: i64,
}

impl PublicationDowRow {
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
                SELECT DAYOFWEEK(`publish_date`) AS `dow`, COUNT(*) AS `comics`
                FROM `Comic`
                WHERE `publish_date` IS NOT NULL
                GROUP BY `dow`
                ORDER BY `dow`
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct ComebackCharacterRow {
    pub id: u16,
    pub name: String,
    pub last_comic: Option<u16>,
    pub return_comic: Option<u16>,
    pub gap_days: Option<i64>,
}

impl ComebackCharacterRow {
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
                    `i`.`id`,
                    `i`.`name`,
                    `g`.`last_comic`,
                    `g`.`return_comic`,
                    `mg`.`max_gap_days` AS `gap_days`
                FROM (
                    SELECT `item_id`, MAX(`gap_days`) AS `max_gap_days`
                    FROM (
                        SELECT
                            `sub`.`item_id`,
                            `sub`.`prev_comic` AS `last_comic`,
                            `sub`.`comic_id` AS `return_comic`,
                            DATEDIFF(`sub`.`return_date`, `sub`.`prev_date`) AS `gap_days`
                        FROM (
                            SELECT
                                `o`.`item_id`,
                                `o`.`comic_id`,
                                `c`.`publish_date` AS `return_date`,
                                LAG(`o`.`comic_id`) OVER (
                                    PARTITION BY `o`.`item_id`
                                    ORDER BY `o`.`comic_id`
                                ) AS `prev_comic`,
                                LAG(`c`.`publish_date`) OVER (
                                    PARTITION BY `o`.`item_id`
                                    ORDER BY `o`.`comic_id`
                                ) AS `prev_date`
                            FROM `Occurrence` `o`
                            JOIN `Comic` `c` ON `o`.`comic_id` = `c`.`id`
                            JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
                            WHERE `i`.`type` = 'cast'
                              AND `c`.`publish_date` IS NOT NULL
                        ) `sub`
                        WHERE `sub`.`prev_comic` IS NOT NULL
                    ) `gaps`
                    GROUP BY `item_id`
                ) `mg`
                JOIN (
                    SELECT `item_id`, `gap_days`,
                           MIN(`last_comic`) AS `last_comic`,
                           MIN(`return_comic`) AS `return_comic`
                    FROM (
                        SELECT
                            `sub`.`item_id`,
                            `sub`.`prev_comic` AS `last_comic`,
                            `sub`.`comic_id` AS `return_comic`,
                            DATEDIFF(`sub`.`return_date`, `sub`.`prev_date`) AS `gap_days`
                        FROM (
                            SELECT
                                `o`.`item_id`,
                                `o`.`comic_id`,
                                `c`.`publish_date` AS `return_date`,
                                LAG(`o`.`comic_id`) OVER (
                                    PARTITION BY `o`.`item_id`
                                    ORDER BY `o`.`comic_id`
                                ) AS `prev_comic`,
                                LAG(`c`.`publish_date`) OVER (
                                    PARTITION BY `o`.`item_id`
                                    ORDER BY `o`.`comic_id`
                                ) AS `prev_date`
                            FROM `Occurrence` `o`
                            JOIN `Comic` `c` ON `o`.`comic_id` = `c`.`id`
                            JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
                            WHERE `i`.`type` = 'cast'
                              AND `c`.`publish_date` IS NOT NULL
                        ) `sub`
                        WHERE `sub`.`prev_comic` IS NOT NULL
                    ) `g_raw`
                    GROUP BY `item_id`, `gap_days`
                ) `g` ON `mg`.`item_id` = `g`.`item_id`
                    AND `mg`.`max_gap_days` = `g`.`gap_days`
                JOIN `Item` `i` ON `mg`.`item_id` = `i`.`id`
                WHERE `mg`.`max_gap_days` >= 90
                ORDER BY `mg`.`max_gap_days` DESC
                LIMIT 50
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct LocationAffinityRow {
    pub location_id: u16,
    pub location_name: String,
    pub character_id: u16,
    pub character_name: String,
    pub comics_together: i64,
}

impl LocationAffinityRow {
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
                    `l`.`id` AS `location_id`,
                    `l`.`name` AS `location_name`,
                    `ch`.`id` AS `character_id`,
                    `ch`.`name` AS `character_name`,
                    COUNT(*) AS `comics_together`
                FROM `Occurrence` `ol`
                JOIN `Occurrence` `oc` ON `ol`.`comic_id` = `oc`.`comic_id`
                JOIN `Item` `l` ON `ol`.`item_id` = `l`.`id`
                JOIN `Item` `ch` ON `oc`.`item_id` = `ch`.`id`
                WHERE `l`.`type` = 'location' AND `ch`.`type` = 'cast'
                GROUP BY `l`.`id`, `ch`.`id`
                ORDER BY `l`.`id`, `comics_together` DESC
            "#,
        )
        .fetch_all(executor)
        .await
    }
}
