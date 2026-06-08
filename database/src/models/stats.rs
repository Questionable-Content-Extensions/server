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
pub struct ComebackLocationRow {
    pub id: u16,
    pub name: String,
    pub last_comic: Option<u16>,
    pub return_comic: Option<u16>,
    pub gap_days: Option<i64>,
}

impl ComebackLocationRow {
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
                            WHERE `i`.`type` = 'location'
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
                            WHERE `i`.`type` = 'location'
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

#[derive(Clone, Copy, Debug, sqlx::FromRow)]
pub struct CrowdedComicRow {
    pub comic_id: u16,
    pub cast_count: i64,
}

impl CrowdedComicRow {
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
                    `c`.`id` AS `comic_id`,
                    COUNT(`i`.`id`) AS `cast_count`
                FROM `Comic` `c`
                JOIN `Occurrence` `o` ON `c`.`id` = `o`.`comic_id`
                JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
                WHERE `i`.`type` = 'cast'
                GROUP BY `c`.`id`
                ORDER BY `cast_count` DESC
                LIMIT 25
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Clone, Copy, Debug, sqlx::FromRow)]
pub struct AvgCastPerYearRow {
    pub year: Option<i32>,
    pub avg_cast_size: Option<f64>,
}

impl AvgCastPerYearRow {
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
                    `sub`.`pub_year` AS `year`,
                    CAST(AVG(`sub`.`cast_count`) AS DOUBLE) AS `avg_cast_size`
                FROM (
                    SELECT
                        YEAR(`c`.`publish_date`) AS `pub_year`,
                        COUNT(`i`.`id`) AS `cast_count`
                    FROM `Comic` `c`
                    LEFT JOIN `Occurrence` `o` ON `c`.`id` = `o`.`comic_id`
                    LEFT JOIN `Item` `i` ON `o`.`item_id` = `i`.`id` AND `i`.`type` = 'cast'
                    WHERE `c`.`publish_date` IS NOT NULL
                    GROUP BY `c`.`id`, YEAR(`c`.`publish_date`)
                ) `sub`
                WHERE `sub`.`pub_year` IS NOT NULL
                GROUP BY `sub`.`pub_year`
                ORDER BY `sub`.`pub_year`
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct LocationYearlyAppearanceRow {
    pub year: Option<i32>,
    pub id: u16,
    pub name: String,
    pub color_red: u8,
    pub color_green: u8,
    pub color_blue: u8,
    pub appearances: i64,
}

impl LocationYearlyAppearanceRow {
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
                WHERE `c`.`publish_date` IS NOT NULL AND `i`.`type` = 'location'
                GROUP BY `year`, `i`.`id`
                ORDER BY `year`, `appearances` DESC
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Clone, Copy, Debug, sqlx::FromRow)]
pub struct PublicationGapRow {
    pub before_comic: Option<u16>,
    pub after_comic: Option<u16>,
    pub gap_days: Option<i32>,
}

impl PublicationGapRow {
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
                    `gaps`.`before_comic`,
                    `gaps`.`after_comic`,
                    `gaps`.`gap_days`
                FROM (
                    SELECT
                        `c`.`id` AS `before_comic`,
                        LEAD(`c`.`id`) OVER (ORDER BY `c`.`publish_date`, `c`.`id`) AS `after_comic`,
                        DATEDIFF(
                            LEAD(`c`.`publish_date`) OVER (ORDER BY `c`.`publish_date`, `c`.`id`),
                            `c`.`publish_date`
                        ) AS `gap_days`
                    FROM `Comic` `c`
                    WHERE `c`.`publish_date` IS NOT NULL
                ) `gaps`
                WHERE `gaps`.`gap_days` IS NOT NULL AND `gaps`.`gap_days` > 6
                ORDER BY `gaps`.`gap_days` DESC
                LIMIT 20
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct DebutDetailRow {
    pub year: Option<i32>,
    pub id: u16,
    pub name: String,
}

impl DebutDetailRow {
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
                    YEAR(MIN(`c`.`publish_date`)) AS `year`,
                    `i`.`id`,
                    `i`.`name`
                FROM `Occurrence` `o`
                JOIN `Comic` `c` ON `o`.`comic_id` = `c`.`id`
                JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
                WHERE `i`.`type` = 'cast' AND `c`.`publish_date` IS NOT NULL
                GROUP BY `i`.`id`
                ORDER BY `year`, `i`.`name`
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Clone, Copy, Debug, sqlx::FromRow)]
pub struct EnsembleRatioRow {
    pub year: Option<i32>,
    pub no_cast: i64,
    pub solo: i64,
    pub small_group: i64,
    pub large_group: i64,
    pub total: i64,
}

impl EnsembleRatioRow {
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
                    `sub`.`pub_year` AS `year`,
                    COUNT(CASE WHEN `sub`.`cast_count` = 0 THEN 1 END) AS `no_cast`,
                    COUNT(CASE WHEN `sub`.`cast_count` = 1 THEN 1 END) AS `solo`,
                    COUNT(CASE WHEN `sub`.`cast_count` BETWEEN 2 AND 4 THEN 1 END) AS `small_group`,
                    COUNT(CASE WHEN `sub`.`cast_count` >= 5 THEN 1 END) AS `large_group`,
                    COUNT(*) AS `total`
                FROM (
                    SELECT
                        YEAR(`c`.`publish_date`) AS `pub_year`,
                        COUNT(`i`.`id`) AS `cast_count`
                    FROM `Comic` `c`
                    LEFT JOIN `Occurrence` `o` ON `c`.`id` = `o`.`comic_id`
                    LEFT JOIN `Item` `i` ON `o`.`item_id` = `i`.`id` AND `i`.`type` = 'cast'
                    WHERE `c`.`publish_date` IS NOT NULL
                    GROUP BY `c`.`id`, YEAR(`c`.`publish_date`)
                ) `sub`
                WHERE `sub`.`pub_year` IS NOT NULL
                GROUP BY `sub`.`pub_year`
                ORDER BY `sub`.`pub_year`
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct CharacterRegularityRow {
    pub id: u16,
    pub name: String,
    pub gap_count: i64,
    pub avg_gap_days: Option<f64>,
    pub stddev_gap_days: Option<f64>,
}

impl CharacterRegularityRow {
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
                    `i`.`id`,
                    `i`.`name`,
                    COUNT(`g`.`gap_days`) AS `gap_count`,
                    CAST(AVG(`g`.`gap_days`) AS DOUBLE) AS `avg_gap_days`,
                    CAST(STDDEV_POP(`g`.`gap_days`) AS DOUBLE) AS `stddev_gap_days`
                FROM (
                    SELECT
                        `o`.`item_id`,
                        DATEDIFF(
                            `c`.`publish_date`,
                            LAG(`c`.`publish_date`) OVER (PARTITION BY `o`.`item_id` ORDER BY `o`.`comic_id`)
                        ) AS `gap_days`
                    FROM `Occurrence` `o`
                    JOIN `Comic` `c` ON `o`.`comic_id` = `c`.`id`
                    JOIN `Item` `i2` ON `o`.`item_id` = `i2`.`id`
                    WHERE `i2`.`type` = 'cast' AND `c`.`publish_date` IS NOT NULL
                ) `g`
                JOIN `Item` `i` ON `g`.`item_id` = `i`.`id`
                WHERE `g`.`gap_days` IS NOT NULL
                GROUP BY `g`.`item_id`
                HAVING COUNT(`g`.`gap_days`) >= 9
                ORDER BY `stddev_gap_days` ASC
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct LocationRegularityRow {
    pub id: u16,
    pub name: String,
    pub gap_count: i64,
    pub avg_gap_days: Option<f64>,
    pub stddev_gap_days: Option<f64>,
}

impl LocationRegularityRow {
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
                    `i`.`id`,
                    `i`.`name`,
                    COUNT(`g`.`gap_days`) AS `gap_count`,
                    CAST(AVG(`g`.`gap_days`) AS DOUBLE) AS `avg_gap_days`,
                    CAST(STDDEV_POP(`g`.`gap_days`) AS DOUBLE) AS `stddev_gap_days`
                FROM (
                    SELECT
                        `o`.`item_id`,
                        DATEDIFF(
                            `c`.`publish_date`,
                            LAG(`c`.`publish_date`) OVER (PARTITION BY `o`.`item_id` ORDER BY `o`.`comic_id`)
                        ) AS `gap_days`
                    FROM `Occurrence` `o`
                    JOIN `Comic` `c` ON `o`.`comic_id` = `c`.`id`
                    JOIN `Item` `i2` ON `o`.`item_id` = `i2`.`id`
                    WHERE `i2`.`type` = 'location' AND `c`.`publish_date` IS NOT NULL
                ) `g`
                JOIN `Item` `i` ON `g`.`item_id` = `i`.`id`
                WHERE `g`.`gap_days` IS NOT NULL
                GROUP BY `g`.`item_id`
                HAVING COUNT(`g`.`gap_days`) >= 9
                ORDER BY `stddev_gap_days` ASC
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct LocationCoOccurrenceRow {
    pub location1_id: u16,
    pub location1_name: String,
    pub location1_appearances: i64,
    pub location2_id: u16,
    pub location2_name: String,
    pub location2_appearances: i64,
    pub comics_together: i64,
}

impl LocationCoOccurrenceRow {
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
                    `l1`.`id` AS `location1_id`,
                    `l1`.`name` AS `location1_name`,
                    `app1`.`total` AS `location1_appearances`,
                    `l2`.`id` AS `location2_id`,
                    `l2`.`name` AS `location2_name`,
                    `app2`.`total` AS `location2_appearances`,
                    COUNT(*) AS `comics_together`
                FROM `Occurrence` `ol1`
                JOIN `Occurrence` `ol2`
                    ON `ol1`.`comic_id` = `ol2`.`comic_id`
                    AND `ol1`.`item_id` < `ol2`.`item_id`
                JOIN `Item` `l1` ON `ol1`.`item_id` = `l1`.`id`
                JOIN `Item` `l2` ON `ol2`.`item_id` = `l2`.`id`
                JOIN (
                    SELECT `item_id`, COUNT(*) AS `total`
                    FROM `Occurrence`
                    GROUP BY `item_id`
                ) `app1` ON `ol1`.`item_id` = `app1`.`item_id`
                JOIN (
                    SELECT `item_id`, COUNT(*) AS `total`
                    FROM `Occurrence`
                    GROUP BY `item_id`
                ) `app2` ON `ol2`.`item_id` = `app2`.`item_id`
                WHERE `l1`.`type` = 'location' AND `l2`.`type` = 'location'
                GROUP BY `ol1`.`item_id`, `ol2`.`item_id`
                ORDER BY `comics_together` DESC
                LIMIT 50
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

impl CoAppearance {
    /// Returns top 100 character pairs sorted by normalized co-appearance score
    /// (comics together / min of each character's total appearances).
    ///
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn top_normalized<'e, 'c: 'e, E>(executor: E) -> sqlx::Result<Vec<Self>>
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
                HAVING COUNT(*) >= 5
                ORDER BY COUNT(*) / LEAST(`app1`.`total`, `app2`.`total`) DESC
                LIMIT 100
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct SocialHubRow {
    pub id: u16,
    pub name: String,
    pub appearances: i64,
    pub distinct_partners: i64,
}

impl SocialHubRow {
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
                    `i`.`id`,
                    `i`.`name`,
                    `app`.`total` AS `appearances`,
                    COUNT(DISTINCT `o2`.`item_id`) AS `distinct_partners`
                FROM `Item` `i`
                JOIN (
                    SELECT `item_id`, COUNT(*) AS `total`
                    FROM `Occurrence`
                    GROUP BY `item_id`
                ) `app` ON `app`.`item_id` = `i`.`id`
                JOIN `Occurrence` `o1` ON `o1`.`item_id` = `i`.`id`
                JOIN `Occurrence` `o2`
                    ON `o2`.`comic_id` = `o1`.`comic_id`
                    AND `o2`.`item_id` != `o1`.`item_id`
                JOIN `Item` `i2` ON `i2`.`id` = `o2`.`item_id` AND `i2`.`type` = 'cast'
                WHERE `i`.`type` = 'cast'
                GROUP BY `i`.`id`
                HAVING COUNT(DISTINCT `o2`.`item_id`) > 0
                ORDER BY `distinct_partners` DESC
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct TrendingItemRow {
    pub id: u16,
    pub name: String,
    pub total_appearances: i64,
    pub recent_appearances: i64,
    pub career_years: Option<f64>,
}

impl TrendingItemRow {
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn cast_trending<'e, 'c: 'e, E>(executor: E) -> sqlx::Result<Vec<Self>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            Self,
            r#"
                SELECT
                    `i`.`id`,
                    `i`.`name`,
                    COUNT(*) AS `total_appearances`,
                    COUNT(CASE WHEN `c`.`publish_date` >= DATE_SUB(
                        (SELECT MAX(`publish_date`) FROM `Comic`), INTERVAL 365 DAY
                    ) THEN 1 END) AS `recent_appearances`,
                    CAST(
                        DATEDIFF(MAX(`c`.`publish_date`), MIN(`c`.`publish_date`)) / 365.25
                    AS DOUBLE) AS `career_years`
                FROM `Item` `i`
                JOIN `Occurrence` `o` ON `o`.`item_id` = `i`.`id`
                JOIN `Comic` `c` ON `o`.`comic_id` = `c`.`id`
                WHERE `i`.`type` = 'cast' AND `c`.`publish_date` IS NOT NULL
                GROUP BY `i`.`id`
                HAVING COUNT(*) >= 5
                ORDER BY `recent_appearances` DESC
            "#,
        )
        .fetch_all(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn location_trending<'e, 'c: 'e, E>(executor: E) -> sqlx::Result<Vec<Self>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            Self,
            r#"
                SELECT
                    `i`.`id`,
                    `i`.`name`,
                    COUNT(*) AS `total_appearances`,
                    COUNT(CASE WHEN `c`.`publish_date` >= DATE_SUB(
                        (SELECT MAX(`publish_date`) FROM `Comic`), INTERVAL 365 DAY
                    ) THEN 1 END) AS `recent_appearances`,
                    CAST(
                        DATEDIFF(MAX(`c`.`publish_date`), MIN(`c`.`publish_date`)) / 365.25
                    AS DOUBLE) AS `career_years`
                FROM `Item` `i`
                JOIN `Occurrence` `o` ON `o`.`item_id` = `i`.`id`
                JOIN `Comic` `c` ON `o`.`comic_id` = `c`.`id`
                WHERE `i`.`type` = 'location' AND `c`.`publish_date` IS NOT NULL
                GROUP BY `i`.`id`
                HAVING COUNT(*) >= 5
                ORDER BY `recent_appearances` DESC
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Copy, Clone, Debug, sqlx::FromRow)]
pub struct CastTurnoverRow {
    pub year: Option<i32>,
    pub new_chars: i64,
    pub continuing_chars: i64,
    pub returning_chars: i64,
    pub dropped_chars: Option<i64>,
}

impl CastTurnoverRow {
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
                WITH `char_yr` AS (
                    SELECT DISTINCT YEAR(`c`.`publish_date`) AS `yr`, `o`.`item_id`
                    FROM `Occurrence` `o`
                    JOIN `Comic` `c` ON `o`.`comic_id` = `c`.`id`
                    JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
                    WHERE `i`.`type` = 'cast' AND `c`.`publish_date` IS NOT NULL
                ),
                `char_first` AS (
                    SELECT `item_id`, MIN(`yr`) AS `first_yr`
                    FROM `char_yr`
                    GROUP BY `item_id`
                )
                SELECT
                    `cur`.`yr` AS `year`,
                    COUNT(CASE WHEN `first`.`first_yr` = `cur`.`yr` THEN 1 END) AS `new_chars`,
                    COUNT(CASE WHEN `first`.`first_yr` < `cur`.`yr` AND `prev`.`item_id` IS NOT NULL THEN 1 END) AS `continuing_chars`,
                    COUNT(CASE WHEN `first`.`first_yr` < `cur`.`yr` AND `prev`.`item_id` IS NULL THEN 1 END) AS `returning_chars`,
                    (
                        SELECT COUNT(DISTINCT `p`.`item_id`)
                        FROM `char_yr` `p`
                        WHERE `p`.`yr` = `cur`.`yr` - 1
                          AND NOT EXISTS (
                            SELECT 1 FROM `char_yr` `n`
                            WHERE `n`.`item_id` = `p`.`item_id` AND `n`.`yr` = `cur`.`yr`
                          )
                    ) AS `dropped_chars`
                FROM `char_yr` `cur`
                JOIN `char_first` `first` ON `first`.`item_id` = `cur`.`item_id`
                LEFT JOIN `char_yr` `prev`
                    ON `prev`.`item_id` = `cur`.`item_id` AND `prev`.`yr` = `cur`.`yr` - 1
                GROUP BY `cur`.`yr`
                ORDER BY `cur`.`yr`
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct CharacterSeasonRow {
    pub id: u16,
    pub name: String,
    pub month: Option<i32>,
    pub appearances: i64,
}

impl CharacterSeasonRow {
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
                    `i`.`id`,
                    `i`.`name`,
                    MONTH(`c`.`publish_date`) AS `month`,
                    COUNT(*) AS `appearances`
                FROM `Item` `i`
                JOIN `Occurrence` `o` ON `o`.`item_id` = `i`.`id`
                JOIN `Comic` `c` ON `o`.`comic_id` = `c`.`id`
                WHERE `i`.`type` = 'cast' AND `c`.`publish_date` IS NOT NULL
                  AND `i`.`id` IN (
                    SELECT `item_id` FROM `Occurrence`
                    GROUP BY `item_id`
                    HAVING COUNT(*) >= 50
                  )
                GROUP BY `i`.`id`, MONTH(`c`.`publish_date`)
                ORDER BY `i`.`id`, `month`
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct BreakoutYearRow {
    pub id: u16,
    pub name: String,
    pub breakout_years: Option<String>,
    pub breakout_count: i64,
    pub avg_per_year: Option<f64>,
}

impl BreakoutYearRow {
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
                    `ranked`.`item_id` AS `id`,
                    `i`.`name`,
                    GROUP_CONCAT(DISTINCT `ranked`.`yr` ORDER BY `ranked`.`yr` SEPARATOR ',') AS `breakout_years`,
                    `ranked`.`cnt` AS `breakout_count`,
                    CAST(`item_totals`.`total_appearances` AS DOUBLE) / NULLIF(`item_totals`.`active_years`, 0) AS `avg_per_year`
                FROM (
                    SELECT
                        `o`.`item_id`,
                        YEAR(`c`.`publish_date`) AS `yr`,
                        COUNT(*) AS `cnt`,
                        RANK() OVER (PARTITION BY `o`.`item_id` ORDER BY COUNT(*) DESC) AS `rnk`
                    FROM `Occurrence` `o`
                    JOIN `Comic` `c` ON `o`.`comic_id` = `c`.`id`
                    JOIN `Item` `i2` ON `o`.`item_id` = `i2`.`id`
                    WHERE `i2`.`type` = 'cast' AND `c`.`publish_date` IS NOT NULL
                    GROUP BY `o`.`item_id`, YEAR(`c`.`publish_date`)
                ) `ranked`
                JOIN `Item` `i` ON `i`.`id` = `ranked`.`item_id`
                JOIN (
                    SELECT
                        `o`.`item_id`,
                        COUNT(*) AS `total_appearances`,
                        COUNT(DISTINCT YEAR(`c`.`publish_date`)) AS `active_years`
                    FROM `Occurrence` `o`
                    JOIN `Comic` `c` ON `o`.`comic_id` = `c`.`id`
                    JOIN `Item` `i2` ON `o`.`item_id` = `i2`.`id`
                    WHERE `i2`.`type` = 'cast' AND `c`.`publish_date` IS NOT NULL
                    GROUP BY `o`.`item_id`
                ) `item_totals` ON `item_totals`.`item_id` = `ranked`.`item_id`
                WHERE `ranked`.`rnk` = 1 AND `item_totals`.`active_years` >= 2
                GROUP BY `ranked`.`item_id`, `ranked`.`cnt`, `item_totals`.`total_appearances`, `item_totals`.`active_years`
                ORDER BY `ranked`.`cnt` DESC
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct CharacterHomeTurfRow {
    pub character_id: u16,
    pub character_name: String,
    pub location_id: u16,
    pub location_name: String,
    pub comics_together: i64,
    pub character_appearances: i64,
}

impl CharacterHomeTurfRow {
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
                    `ranked`.`item_id` AS `character_id`,
                    `ic`.`name` AS `character_name`,
                    `ranked`.`loc_id` AS `location_id`,
                    `il`.`name` AS `location_name`,
                    `ranked`.`cnt` AS `comics_together`,
                    `char_app`.`total` AS `character_appearances`
                FROM (
                    SELECT
                        `oc`.`item_id`,
                        `ol`.`item_id` AS `loc_id`,
                        COUNT(*) AS `cnt`,
                        RANK() OVER (PARTITION BY `oc`.`item_id` ORDER BY COUNT(*) DESC) AS `rnk`
                    FROM `Occurrence` `oc`
                    JOIN `Item` `ic2` ON `oc`.`item_id` = `ic2`.`id` AND `ic2`.`type` = 'cast'
                    JOIN `Occurrence` `ol` ON `ol`.`comic_id` = `oc`.`comic_id`
                    JOIN `Item` `il2` ON `ol`.`item_id` = `il2`.`id` AND `il2`.`type` = 'location'
                    GROUP BY `oc`.`item_id`, `ol`.`item_id`
                ) `ranked`
                JOIN `Item` `ic` ON `ic`.`id` = `ranked`.`item_id`
                JOIN `Item` `il` ON `il`.`id` = `ranked`.`loc_id`
                JOIN (
                    SELECT `item_id`, COUNT(*) AS `total`
                    FROM `Occurrence`
                    JOIN `Item` ON `Occurrence`.`item_id` = `Item`.`id` AND `Item`.`type` = 'cast'
                    GROUP BY `item_id`
                ) `char_app` ON `char_app`.`item_id` = `ranked`.`item_id`
                WHERE `ranked`.`rnk` = 1 AND `char_app`.`total` >= 10
                ORDER BY CAST(`ranked`.`cnt` AS DOUBLE) / `char_app`.`total` DESC, `ranked`.`item_id` ASC
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Copy, Clone, Debug, sqlx::FromRow)]
pub struct PairEvolutionRow {
    pub year: Option<i32>,
    pub comics_together: i64,
}

impl PairEvolutionRow {
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn for_pair<'e, 'c: 'e, E>(
        executor: E,
        char1: u16,
        char2: u16,
    ) -> sqlx::Result<Vec<Self>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            Self,
            r#"
                SELECT
                    YEAR(`c`.`publish_date`) AS `year`,
                    COUNT(*) AS `comics_together`
                FROM `Occurrence` `o1`
                JOIN `Occurrence` `o2`
                    ON `o1`.`comic_id` = `o2`.`comic_id`
                    AND `o2`.`item_id` = ?
                JOIN `Comic` `c` ON `o1`.`comic_id` = `c`.`id`
                WHERE `o1`.`item_id` = ? AND `c`.`publish_date` IS NOT NULL
                GROUP BY YEAR(`c`.`publish_date`)
                ORDER BY `year`
            "#,
            char2,
            char1,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct LonerIndexRow {
    pub id: u16,
    pub name: String,
    pub appearances: i64,
    pub avg_co_cast: Option<f64>,
}

impl LonerIndexRow {
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
                    `i`.`id`,
                    `i`.`name`,
                    COUNT(*) AS `appearances`,
                    CAST(AVG(GREATEST(0, `cast_count`.`cnt` - 1)) AS DOUBLE) AS `avg_co_cast`
                FROM `Item` `i`
                JOIN `Occurrence` `o` ON `o`.`item_id` = `i`.`id`
                JOIN (
                    SELECT `o2`.`comic_id`, COUNT(*) AS `cnt`
                    FROM `Occurrence` `o2`
                    JOIN `Item` `i2` ON `o2`.`item_id` = `i2`.`id` AND `i2`.`type` = 'cast'
                    GROUP BY `o2`.`comic_id`
                ) `cast_count` ON `cast_count`.`comic_id` = `o`.`comic_id`
                WHERE `i`.`type` = 'cast'
                GROUP BY `i`.`id`
                HAVING COUNT(*) >= 10
                ORDER BY `avg_co_cast` ASC
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct NeverMetRow {
    pub character1_id: u16,
    pub character1_name: String,
    pub character1_appearances: i64,
    pub character2_id: u16,
    pub character2_name: String,
    pub character2_appearances: i64,
    pub comics_together: Option<i64>,
}

impl NeverMetRow {
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
                    `c1`.`id` AS `character1_id`,
                    `c1`.`name` AS `character1_name`,
                    `c1`.`appearances` AS `character1_appearances`,
                    `c2`.`id` AS `character2_id`,
                    `c2`.`name` AS `character2_name`,
                    `c2`.`appearances` AS `character2_appearances`,
                    COALESCE(`ca`.`comics_together`, 0) AS `comics_together`
                FROM (
                    SELECT `i`.`id`, `i`.`name`, COUNT(*) AS `appearances`
                    FROM `Item` `i`
                    JOIN `Occurrence` `o` ON `o`.`item_id` = `i`.`id`
                    WHERE `i`.`type` = 'cast'
                    GROUP BY `i`.`id`
                    ORDER BY `appearances` DESC
                    LIMIT 100
                ) `c1`
                JOIN (
                    SELECT `i`.`id`, `i`.`name`, COUNT(*) AS `appearances`
                    FROM `Item` `i`
                    JOIN `Occurrence` `o` ON `o`.`item_id` = `i`.`id`
                    WHERE `i`.`type` = 'cast'
                    GROUP BY `i`.`id`
                    ORDER BY `appearances` DESC
                    LIMIT 100
                ) `c2` ON `c1`.`id` < `c2`.`id`
                LEFT JOIN (
                    SELECT
                        `o1`.`item_id` AS `char1_id`,
                        `o2`.`item_id` AS `char2_id`,
                        COUNT(*) AS `comics_together`
                    FROM `Occurrence` `o1`
                    JOIN `Occurrence` `o2`
                        ON `o1`.`comic_id` = `o2`.`comic_id`
                        AND `o1`.`item_id` < `o2`.`item_id`
                    GROUP BY `o1`.`item_id`, `o2`.`item_id`
                ) `ca` ON `ca`.`char1_id` = `c1`.`id` AND `ca`.`char2_id` = `c2`.`id`
                WHERE COALESCE(`ca`.`comics_together`, 0) <= 2
                ORDER BY `c1`.`appearances` + `c2`.`appearances` DESC
                LIMIT 200
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Copy, Clone, Debug, sqlx::FromRow)]
pub struct PublishTimeRow {
    pub year: Option<i32>,
    pub hour: Option<i32>,
    pub comics: i64,
}

impl PublishTimeRow {
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
                    YEAR(`publish_date`) AS `year`,
                    HOUR(`publish_date`) AS `hour`,
                    COUNT(*) AS `comics`
                FROM `Comic`
                WHERE `publish_date` IS NOT NULL
                GROUP BY YEAR(`publish_date`), HOUR(`publish_date`)
                ORDER BY `year`, `hour`
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Copy, Clone, Debug, sqlx::FromRow)]
pub struct ScheduleEvolutionRow {
    pub year: Option<i32>,
    pub dow: Option<i32>,
    pub comics: i64,
}

impl ScheduleEvolutionRow {
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
                    YEAR(`publish_date`) AS `year`,
                    DAYOFWEEK(`publish_date`) AS `dow`,
                    COUNT(*) AS `comics`
                FROM `Comic`
                WHERE `publish_date` IS NOT NULL
                GROUP BY YEAR(`publish_date`), DAYOFWEEK(`publish_date`)
                ORDER BY `year`, `dow`
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct PublishedDateRow {
    pub pub_date: Option<String>,
}

impl PublishedDateRow {
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn all_distinct<'e, 'c: 'e, E>(executor: E) -> sqlx::Result<Vec<Self>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            Self,
            r#"
                SELECT DISTINCT DATE_FORMAT(DATE(`publish_date`), '%Y-%m-%d') AS `pub_date`
                FROM `Comic`
                WHERE `publish_date` IS NOT NULL
                ORDER BY `pub_date`
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Copy, Clone, Debug, sqlx::FromRow)]
pub struct MonthlyHeatmapRow {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub comics: i64,
}

impl MonthlyHeatmapRow {
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
                    YEAR(`publish_date`) AS `year`,
                    MONTH(`publish_date`) AS `month`,
                    COUNT(*) AS `comics`
                FROM `Comic`
                WHERE `publish_date` IS NOT NULL
                GROUP BY YEAR(`publish_date`), MONTH(`publish_date`)
                ORDER BY `year`, `month`
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct MilestoneComicRow {
    pub comic_id: u16,
    pub title: String,
    pub pub_date: Option<String>,
    pub is_guest_comic: u8,
    pub is_non_canon: u8,
}

impl MilestoneComicRow {
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
                    `id` AS `comic_id`,
                    `title`,
                    DATE_FORMAT(`publish_date`, '%Y-%m-%d') AS `pub_date`,
                    `is_guest_comic`,
                    `is_non_canon`
                FROM `Comic`
                WHERE `id` = 1
                   OR (`id` >= 100 AND `id` <= 1000 AND `id` % 100 = 0)
                   OR (`id` > 1000 AND `id` % 500 = 0)
                ORDER BY `id`
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct LocationSeasonRow {
    pub id: u16,
    pub name: String,
    pub month: Option<i32>,
    pub appearances: i64,
}

impl LocationSeasonRow {
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
                    `i`.`id`,
                    `i`.`name`,
                    MONTH(`c`.`publish_date`) AS `month`,
                    COUNT(*) AS `appearances`
                FROM `Item` `i`
                JOIN `Occurrence` `o` ON `o`.`item_id` = `i`.`id`
                JOIN `Comic` `c` ON `o`.`comic_id` = `c`.`id`
                WHERE `i`.`type` = 'location' AND `c`.`publish_date` IS NOT NULL
                  AND `i`.`id` IN (
                    SELECT `item_id` FROM `Occurrence`
                    GROUP BY `item_id`
                    HAVING COUNT(*) >= 50
                  )
                GROUP BY `i`.`id`, MONTH(`c`.`publish_date`)
                ORDER BY `i`.`id`, `month`
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct LocationBreakoutYearRow {
    pub id: u16,
    pub name: String,
    pub breakout_years: Option<String>,
    pub breakout_count: i64,
    pub avg_per_year: Option<f64>,
}

impl LocationBreakoutYearRow {
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
                    `ranked`.`item_id` AS `id`,
                    `i`.`name`,
                    GROUP_CONCAT(DISTINCT `ranked`.`yr` ORDER BY `ranked`.`yr` SEPARATOR ',') AS `breakout_years`,
                    `ranked`.`cnt` AS `breakout_count`,
                    CAST(`item_totals`.`total_appearances` AS DOUBLE) / NULLIF(`item_totals`.`active_years`, 0) AS `avg_per_year`
                FROM (
                    SELECT
                        `o`.`item_id`,
                        YEAR(`c`.`publish_date`) AS `yr`,
                        COUNT(*) AS `cnt`,
                        RANK() OVER (PARTITION BY `o`.`item_id` ORDER BY COUNT(*) DESC) AS `rnk`
                    FROM `Occurrence` `o`
                    JOIN `Comic` `c` ON `o`.`comic_id` = `c`.`id`
                    JOIN `Item` `i2` ON `o`.`item_id` = `i2`.`id`
                    WHERE `i2`.`type` = 'location' AND `c`.`publish_date` IS NOT NULL
                    GROUP BY `o`.`item_id`, YEAR(`c`.`publish_date`)
                ) `ranked`
                JOIN `Item` `i` ON `i`.`id` = `ranked`.`item_id`
                JOIN (
                    SELECT
                        `o`.`item_id`,
                        COUNT(*) AS `total_appearances`,
                        COUNT(DISTINCT YEAR(`c`.`publish_date`)) AS `active_years`
                    FROM `Occurrence` `o`
                    JOIN `Comic` `c` ON `o`.`comic_id` = `c`.`id`
                    JOIN `Item` `i2` ON `o`.`item_id` = `i2`.`id`
                    WHERE `i2`.`type` = 'location' AND `c`.`publish_date` IS NOT NULL
                    GROUP BY `o`.`item_id`
                ) `item_totals` ON `item_totals`.`item_id` = `ranked`.`item_id`
                WHERE `ranked`.`rnk` = 1 AND `item_totals`.`active_years` >= 2
                GROUP BY `ranked`.`item_id`, `ranked`.`cnt`, `item_totals`.`total_appearances`, `item_totals`.`active_years`
                ORDER BY `ranked`.`cnt` DESC
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct LocationSocialHubRow {
    pub id: u16,
    pub name: String,
    pub appearances: i64,
    pub distinct_characters: i64,
}

impl LocationSocialHubRow {
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
                    `i`.`id`,
                    `i`.`name`,
                    `app`.`total` AS `appearances`,
                    COUNT(DISTINCT `o2`.`item_id`) AS `distinct_characters`
                FROM `Item` `i`
                JOIN (
                    SELECT `item_id`, COUNT(*) AS `total`
                    FROM `Occurrence`
                    GROUP BY `item_id`
                ) `app` ON `app`.`item_id` = `i`.`id`
                JOIN `Occurrence` `o1` ON `o1`.`item_id` = `i`.`id`
                JOIN `Occurrence` `o2`
                    ON `o2`.`comic_id` = `o1`.`comic_id`
                    AND `o2`.`item_id` != `o1`.`item_id`
                JOIN `Item` `i2` ON `i2`.`id` = `o2`.`item_id` AND `i2`.`type` = 'cast'
                WHERE `i`.`type` = 'location'
                GROUP BY `i`.`id`
                HAVING COUNT(DISTINCT `o2`.`item_id`) > 0
                ORDER BY `distinct_characters` DESC
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[derive(Copy, Clone, Debug, sqlx::FromRow)]
pub struct LocationTurnoverRow {
    pub year: Option<i32>,
    pub new_locations: i64,
    pub continuing_locations: i64,
    pub returning_locations: i64,
    pub dropped_locations: Option<i64>,
}

impl LocationTurnoverRow {
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
                WITH `loc_yr` AS (
                    SELECT DISTINCT YEAR(`c`.`publish_date`) AS `yr`, `o`.`item_id`
                    FROM `Occurrence` `o`
                    JOIN `Comic` `c` ON `o`.`comic_id` = `c`.`id`
                    JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
                    WHERE `i`.`type` = 'location' AND `c`.`publish_date` IS NOT NULL
                ),
                `loc_first` AS (
                    SELECT `item_id`, MIN(`yr`) AS `first_yr`
                    FROM `loc_yr`
                    GROUP BY `item_id`
                )
                SELECT
                    `cur`.`yr` AS `year`,
                    COUNT(CASE WHEN `first`.`first_yr` = `cur`.`yr` THEN 1 END) AS `new_locations`,
                    COUNT(CASE WHEN `first`.`first_yr` < `cur`.`yr` AND `prev`.`item_id` IS NOT NULL THEN 1 END) AS `continuing_locations`,
                    COUNT(CASE WHEN `first`.`first_yr` < `cur`.`yr` AND `prev`.`item_id` IS NULL THEN 1 END) AS `returning_locations`,
                    (
                        SELECT COUNT(DISTINCT `p`.`item_id`)
                        FROM `loc_yr` `p`
                        WHERE `p`.`yr` = `cur`.`yr` - 1
                          AND NOT EXISTS (
                            SELECT 1 FROM `loc_yr` `n`
                            WHERE `n`.`item_id` = `p`.`item_id` AND `n`.`yr` = `cur`.`yr`
                          )
                    ) AS `dropped_locations`
                FROM `loc_yr` `cur`
                JOIN `loc_first` `first` ON `first`.`item_id` = `cur`.`item_id`
                LEFT JOIN `loc_yr` `prev`
                    ON `prev`.`item_id` = `cur`.`item_id` AND `prev`.`yr` = `cur`.`yr` - 1
                GROUP BY `cur`.`yr`
                ORDER BY `cur`.`yr`
            "#,
        )
        .fetch_all(executor)
        .await
    }
}
