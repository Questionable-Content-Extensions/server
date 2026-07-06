use chrono::{DateTime, NaiveDateTime, Utc};
use futures::TryStreamExt;

use crate::models::ItemType;

#[expect(
    clippy::struct_field_names,
    reason = "field names match the database column names"
)]
#[derive(Debug)]
pub struct Comic {
    pub id: u16,
    pub image_type: i32,
    pub is_guest_comic: u8,
    pub is_non_canon: u8,
    pub has_no_cast: u8,
    pub has_no_location: u8,
    pub has_no_storyline: u8,
    pub has_no_title: u8,
    pub has_no_tagline: u8,
    pub title: String,
    pub tagline: Option<String>,
    pub publish_date: Option<NaiveDateTime>,
    pub is_accurate_publish_date: u8,
    pub hidden: u8,
}

impl Comic {
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn count<'e, 'c: 'e, E>(executor: E) -> sqlx::Result<i64>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT COUNT(*) FROM `Comic`
            "#,
        )
        .fetch_one(executor)
        .await
    }

    /// Latest known (non-hidden) comic id, used as the fallback right bound
    /// when computing visual segments for an open-ended storyline.
    ///
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn latest_id<'e, 'c: 'e, E>(executor: E) -> sqlx::Result<Option<u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT MAX(`id`) FROM `Comic` WHERE NOT `hidden`
            "#,
        )
        .fetch_one(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor, map))]
    pub async fn all_with_mapping<'e, 'c: 'e, E, T, F>(
        executor: E,
        include_guest_comics: Option<bool>,
        include_non_canon_comics: Option<bool>,
        map: F,
    ) -> sqlx::Result<Vec<T>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
        F: FnMut(Self) -> T,
    {
        // Hidden (advance) comics are never listed here, even for editors —
        // normal browsing must never surface them; only a direct by-id fetch
        // or the dedicated pending-advance-comics listing may reveal them.
        sqlx::query_as!(
            Self,
            r#"
                SELECT * FROM `Comic`
                WHERE (? is NULL OR `is_guest_comic` = ?)
                    AND (? is NULL OR `is_non_canon` = ?)
                    AND NOT `hidden`
                ORDER BY `id` ASC
            "#,
            include_guest_comics,
            include_guest_comics,
            include_non_canon_comics,
            include_non_canon_comics,
        )
        .fetch(executor)
        .map_ok(map)
        .try_collect()
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor, map))]
    pub async fn all_hidden_with_mapping<'e, 'c: 'e, E, T, F>(
        executor: E,
        map: F,
    ) -> sqlx::Result<Vec<T>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
        F: FnMut(Self) -> T,
    {
        sqlx::query_as!(
            Self,
            r#"
                SELECT * FROM `Comic`
                WHERE `hidden`
                ORDER BY `id` ASC
            "#,
        )
        .fetch(executor)
        .map_ok(map)
        .try_collect()
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn ensure_exists_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                INSERT IGNORE INTO `Comic`
                    (`id`)
                VALUES
                    (?)
            "#,
            id,
        )
        .execute(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn exists_by_id<'e, 'c: 'e, E>(executor: E, id: u16) -> sqlx::Result<bool>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT COUNT(1) FROM `Comic`
                WHERE
                    `id` = ?
            "#,
            id,
        )
        .fetch_one(executor)
        .await
        .map(|c| c == 1)
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn by_id<'e, 'c: 'e, E>(executor: E, id: u16) -> sqlx::Result<Option<Self>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            Self,
            r#"
                SELECT * FROM `Comic`
                WHERE `id` = ?
            "#,
            id
        )
        .fetch_optional(executor)
        .await
    }

    /// Fetches a comic by ID together with its previous/next IDs and news text in one query.
    ///
    /// Returns `None` when the comic does not exist.  The `prev_id` / `next_id` fields
    /// respect the same guest/non-canon filters as the standalone `previous_id`/`next_id`
    /// functions.  `news` is `None` when no news row exists for the comic.
    ///
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn by_id_with_navigation_and_news<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        include_guest_comics: Option<bool>,
        include_non_canon_comics: Option<bool>,
        include_hidden: bool,
    ) -> sqlx::Result<Option<ComicWithNavigationAndNews>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        // `include_hidden` only ever gates the direct by-id lookup below (so an
        // editor with a known id — e.g. from the pending-advance-comics list —
        // can fetch a hidden comic). The prev_id/next_id navigation subqueries
        // always exclude hidden comics, even for editors: normal prev/next
        // browsing must never surface a comic that isn't published yet.
        sqlx::query_as!(
            ComicWithNavigationAndNews,
            r#"
                SELECT
                    `c`.`id`,
                    `c`.`image_type`,
                    `c`.`is_guest_comic`,
                    `c`.`is_non_canon`,
                    `c`.`has_no_cast`,
                    `c`.`has_no_location`,
                    `c`.`has_no_storyline`,
                    `c`.`has_no_title`,
                    `c`.`has_no_tagline`,
                    `c`.`title`,
                    `c`.`tagline`,
                    `c`.`publish_date`,
                    `c`.`is_accurate_publish_date`,
                    (SELECT `id` FROM `Comic`
                     WHERE `id` < `c`.`id`
                       AND (? IS NULL OR `is_guest_comic` = ?)
                       AND (? IS NULL OR `is_non_canon` = ?)
                       AND NOT `hidden`
                     ORDER BY `id` DESC LIMIT 1) AS `prev_id`,
                    (SELECT `id` FROM `Comic`
                     WHERE `id` > `c`.`id`
                       AND (? IS NULL OR `is_guest_comic` = ?)
                       AND (? IS NULL OR `is_non_canon` = ?)
                       AND NOT `hidden`
                     ORDER BY `id` ASC LIMIT 1) AS `next_id`,
                    `n`.`news` AS `news`
                FROM `Comic` `c`
                LEFT JOIN `News` `n` ON `n`.`comic_id` = `c`.`id`
                WHERE `c`.`id` = ? AND (? OR NOT `c`.`hidden`)
            "#,
            include_guest_comics,
            include_guest_comics,
            include_non_canon_comics,
            include_non_canon_comics,
            include_guest_comics,
            include_guest_comics,
            include_non_canon_comics,
            include_non_canon_comics,
            id,
            include_hidden,
        )
        .fetch_optional(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor, map))]
    pub async fn all_with_item_id_mapped<'e, 'c: 'e, E, T, F>(
        executor: E,
        item_id: u16,
        map: F,
    ) -> sqlx::Result<Vec<T>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
        F: FnMut(Self) -> T,
    {
        sqlx::query_as!(
            Self,
            r#"
                SELECT c.* FROM `Comic` c
                JOIN `Occurrence` o on o.`comic_id` = c.`id`
                WHERE o.`item_id` = ?
                ORDER BY c.`id` ASC
            "#,
            item_id
        )
        .fetch(executor)
        .map_ok(map)
        .try_collect()
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn random_comic_id_with_item_id<'e, 'c: 'e, E>(
        executor: E,
        item_id: u16,
        current_comic: u16,
        include_guest_comics: Option<bool>,
        include_non_canon_comics: Option<bool>,
    ) -> sqlx::Result<Option<u16>>
    where
        E: 'e + Copy + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        let count = sqlx::query_scalar!(
            r#"
                SELECT COUNT(*) FROM `Comic` c
                JOIN `Occurrence` o ON o.`comic_id` = c.`id`
                WHERE o.`item_id` = ?
                    AND c.`id` != ?
                    AND (? IS NULL OR c.`is_guest_comic` = ?)
                    AND (? IS NULL OR c.`is_non_canon` = ?)
            "#,
            item_id,
            current_comic,
            include_guest_comics,
            include_guest_comics,
            include_non_canon_comics,
            include_non_canon_comics,
        )
        .fetch_one(executor)
        .await?;

        if count == 0 {
            return Ok(None);
        }

        let offset = rand::random::<u64>() % u64::try_from(count).unwrap_or(u64::MAX);

        sqlx::query_scalar!(
            r#"
                SELECT c.`id` FROM `Comic` c
                JOIN `Occurrence` o ON o.`comic_id` = c.`id`
                WHERE o.`item_id` = ?
                    AND c.`id` != ?
                    AND (? IS NULL OR c.`is_guest_comic` = ?)
                    AND (? IS NULL OR c.`is_non_canon` = ?)
                ORDER BY c.`id`
                LIMIT ?, 1
            "#,
            item_id,
            current_comic,
            include_guest_comics,
            include_guest_comics,
            include_non_canon_comics,
            include_non_canon_comics,
            offset,
        )
        .fetch_optional(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn previous_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        include_guest_comics: Option<bool>,
        include_non_canon_comics: Option<bool>,
    ) -> sqlx::Result<Option<u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT `id`
                FROM `Comic`
                WHERE `id` < ?
                    AND (? is NULL OR `is_guest_comic` = ?)
                    AND (? is NULL OR `is_non_canon` = ?)
                ORDER BY `id` DESC
            "#,
            id,
            include_guest_comics,
            include_guest_comics,
            include_non_canon_comics,
            include_non_canon_comics,
        )
        .fetch_optional(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn next_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        include_guest_comics: Option<bool>,
        include_non_canon_comics: Option<bool>,
    ) -> sqlx::Result<Option<u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT `id`
                FROM `Comic`
                WHERE `id` > ?
                    AND (? is NULL OR `is_guest_comic` = ?)
                    AND (? is NULL OR `is_non_canon` = ?)
                ORDER BY `id` ASC
            "#,
            id,
            include_guest_comics,
            include_guest_comics,
            include_non_canon_comics,
            include_non_canon_comics,
        )
        .fetch_optional(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn publish_date_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
    ) -> sqlx::Result<Option<NaiveDateTime>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT `publish_date` FROM `Comic` WHERE `id` = ?
            "#,
            id
        )
        .fetch_one(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn update_publish_date_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        publish_date: DateTime<Utc>,
        is_accurate_publish_date: bool,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                UPDATE `Comic`
                SET
                    `publish_date` = ?,
                    `is_accurate_publish_date` = ?
                WHERE
                    `id` = ?
            "#,
            publish_date.naive_utc(),
            is_accurate_publish_date,
            id,
        )
        .execute(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn update_is_guest_comic_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        is_guest_comic: bool,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                UPDATE `Comic`
                SET
                    `is_guest_comic` = ?
                WHERE
                    `id` = ?
            "#,
            is_guest_comic,
            id,
        )
        .execute(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn update_is_non_canon_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        is_non_canon: bool,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                UPDATE `Comic`
                SET
                    `is_non_canon` = ?
                WHERE
                    `id` = ?
            "#,
            is_non_canon,
            id,
        )
        .execute(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn update_has_no_cast_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        has_no_cast: bool,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                UPDATE `Comic`
                SET
                    `has_no_cast` = ?
                WHERE
                    `id` = ?
            "#,
            has_no_cast,
            id,
        )
        .execute(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn update_has_no_location_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        has_no_location: bool,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                UPDATE `Comic`
                SET
                    `has_no_location` = ?
                WHERE
                    `id` = ?
            "#,
            has_no_location,
            id,
        )
        .execute(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn update_has_no_storyline_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        has_no_storyline: bool,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                UPDATE `Comic`
                SET
                    `has_no_storyline` = ?
                WHERE
                    `id` = ?
            "#,
            has_no_storyline,
            id,
        )
        .execute(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn update_has_no_title_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        has_no_title: bool,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                UPDATE `Comic`
                SET
                    `has_no_title` = ?
                WHERE
                    `id` = ?
            "#,
            has_no_title,
            id,
        )
        .execute(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn update_has_no_tagline_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        has_no_tagline: bool,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                UPDATE `Comic`
                SET
                    `has_no_tagline` = ?
                WHERE
                    `id` = ?
            "#,
            has_no_tagline,
            id,
        )
        .execute(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn first_and_last_missing_tagline<'e, 'c: 'e, E>(
        executor: E,
    ) -> sqlx::Result<(Option<u16>, Option<u16>)>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            FirstLast,
            r#"
                SELECT
                    MIN(`c`.`id`) as `first`,
                    MAX(`c`.`id`) as `last`
                FROM `Comic` `c`
                WHERE (`c`.`tagline` IS NULL or NULLIF(`c`.`tagline`, '') IS NULL)
                    AND NOT `c`.`has_no_tagline`
                    AND `c`.`id` > 3132
            "#
        )
        .fetch_optional(executor)
        .await
        .map(|ofl| ofl.map_or((None, None), |fl| (fl.first, fl.last)))
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn previous_missing_tagline_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
    ) -> sqlx::Result<Option<u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT `c`.`id`
                FROM `Comic` `c`
                WHERE (`c`.`tagline` IS NULL OR NULLIF(`c`.`tagline`, '') IS NULL)
                    AND NOT `c`.`has_no_tagline`
                    AND `c`.`id` < ?
                    AND `c`.`id` > 3132
                ORDER BY `c`.`id` DESC
                LIMIT 1
            "#,
            id
        )
        .fetch_optional(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn next_missing_tagline_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
    ) -> sqlx::Result<Option<u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT `c`.`id`
                FROM `Comic` `c`
                WHERE (`c`.`tagline` IS NULL OR NULLIF(c.`tagline`, '') IS NULL)
                    AND NOT `c`.`has_no_tagline`
                    AND `c`.`id` > ?
                    AND `c`.`id` > 3132
                ORDER BY `c`.`id` ASC
                LIMIT 1
            "#,
            id
        )
        .fetch_optional(executor)
        .await
    }

    // ---

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn first_and_last_missing_title<'e, 'c: 'e, E>(
        executor: E,
    ) -> sqlx::Result<(Option<u16>, Option<u16>)>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            FirstLast,
            r#"
                SELECT
                    MIN(`c`.`id`) as `first`,
                    MAX(`c`.`id`) as `last`
                FROM `Comic` `c`
                WHERE (`c`.`title` IS NULL or NULLIF(`c`.`title`, '') IS NULL)
                    AND NOT `c`.`has_no_title`
            "#
        )
        .fetch_optional(executor)
        .await
        .map(|ofl| ofl.map_or((None, None), |fl| (fl.first, fl.last)))
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn previous_missing_title_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
    ) -> sqlx::Result<Option<u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT `c`.`id`
                FROM `Comic` `c`
                WHERE (`c`.`title` IS NULL OR NULLIF(`c`.`title`, '') IS NULL)
                    AND NOT `c`.`has_no_title`
                    AND `c`.`id` < ?
                ORDER BY `c`.`id` DESC
                LIMIT 1
            "#,
            id
        )
        .fetch_optional(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn next_missing_title_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
    ) -> sqlx::Result<Option<u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT `c`.`id`
                FROM `Comic` `c`
                WHERE (`c`.`title` IS NULL OR NULLIF(`c`.`title`, '') IS NULL)
                    AND NOT `c`.`has_no_title`
                    AND `c`.`id` > ?
                ORDER BY `c`.`id` ASC
                LIMIT 1
            "#,
            id
        )
        .fetch_optional(executor)
        .await
    }

    // ---

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn first_missing_items_by_type<'e, 'c: 'e, E>(
        executor: E,
        r#type: ItemType,
    ) -> sqlx::Result<Option<u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        let r#type = r#type.as_str();
        sqlx::query_scalar!(
            r#"
                SELECT `c`.`id`
                FROM `Comic` `c`
                WHERE NOT EXISTS (
                    SELECT 1 FROM `Occurrence` `o`
                    JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
                    WHERE `i`.`type` = ?
                    AND `o`.`comic_id` = `c`.`id`
                )
                AND (? <> 'cast' OR NOT `c`.`has_no_cast`)
                AND (? <> 'location' OR NOT `c`.`has_no_location`)
                AND (? <> 'storyline' OR NOT `c`.`has_no_storyline`)
                ORDER BY `c`.`id` ASC
                LIMIT 1
            "#,
            r#type,
            r#type,
            r#type,
            r#type
        )
        .fetch_optional(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn previous_missing_items_by_id_and_type<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        r#type: ItemType,
    ) -> sqlx::Result<Option<u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        let r#type = r#type.as_str();
        sqlx::query_scalar!(
            r#"
                SELECT `c`.`id`
                FROM `Comic` `c`
                WHERE NOT EXISTS (
                    SELECT 1 FROM `Occurrence` `o`
                    JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
                    WHERE `i`.`type` = ?
                    AND `o`.`comic_id` = `c`.`id`
                )
                AND `c`.`id` < ?
                AND (? <> 'cast' OR NOT `c`.`has_no_cast`)
                AND (? <> 'location' OR NOT `c`.`has_no_location`)
                AND (? <> 'storyline' OR NOT `c`.`has_no_storyline`)
                ORDER BY `c`.`id` DESC
                LIMIT 1
            "#,
            r#type,
            id,
            r#type,
            r#type,
            r#type
        )
        .fetch_optional(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn next_missing_items_by_id_and_type<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        r#type: ItemType,
    ) -> sqlx::Result<Option<u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        let r#type = r#type.as_str();
        sqlx::query_scalar!(
            r#"
                SELECT `c`.`id`
                FROM `Comic` `c`
                WHERE NOT EXISTS (
                    SELECT 1 FROM `Occurrence` `o`
                    JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
                    WHERE `i`.`type` = ?
                    AND `o`.`comic_id` = `c`.`id`
                )
                AND `c`.`id` > ?
                AND (? <> 'cast' OR NOT `c`.`has_no_cast`)
                AND (? <> 'location' OR NOT `c`.`has_no_location`)
                AND (? <> 'storyline' OR NOT `c`.`has_no_storyline`)
                ORDER BY `c`.`id` ASC
                LIMIT 1
            "#,
            r#type,
            id,
            r#type,
            r#type,
            r#type
        )
        .fetch_optional(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn last_missing_items_by_type<'e, 'c: 'e, E>(
        executor: E,
        r#type: ItemType,
    ) -> sqlx::Result<Option<u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        let r#type = r#type.as_str();
        sqlx::query_scalar!(
            r#"
                SELECT `c`.`id`
                FROM `Comic` `c`
                WHERE NOT EXISTS (
                    SELECT 1 FROM `Occurrence` `o`
                    JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
                    WHERE `i`.`type` = ?
                    AND `o`.`comic_id` = `c`.`id`
                )
                AND (? <> 'cast' OR NOT `c`.`has_no_cast`)
                AND (? <> 'location' OR NOT `c`.`has_no_location`)
                AND (? <> 'storyline' OR NOT `c`.`has_no_storyline`)
                ORDER BY `c`.`id` DESC
                LIMIT 1
            "#,
            r#type,
            r#type,
            r#type,
            r#type
        )
        .fetch_optional(executor)
        .await
    }

    /// Returns first, last, previous, and next comic IDs that are missing a cast item,
    /// in a single query against the `v_comics_missing_cast` view.
    ///
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn missing_cast_navigation<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
    ) -> sqlx::Result<NavigationResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            NavigationResult,
            r#"
                SELECT
                    MIN(`id`) AS "first?: u16",
                    MAX(`id`) AS "last?: u16",
                    MAX(CASE WHEN `id` < ? THEN `id` END) AS "previous?: u16",
                    MIN(CASE WHEN `id` > ? THEN `id` END) AS "next?: u16"
                FROM `v_comics_missing_cast`
            "#,
            comic_id,
            comic_id,
        )
        .fetch_one(executor)
        .await
    }

    /// Returns first, last, previous, and next comic IDs that are missing a location item,
    /// in a single query against the `v_comics_missing_location` view.
    ///
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn missing_location_navigation<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
    ) -> sqlx::Result<NavigationResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            NavigationResult,
            r#"
                SELECT
                    MIN(`id`) AS "first?: u16",
                    MAX(`id`) AS "last?: u16",
                    MAX(CASE WHEN `id` < ? THEN `id` END) AS "previous?: u16",
                    MIN(CASE WHEN `id` > ? THEN `id` END) AS "next?: u16"
                FROM `v_comics_missing_location`
            "#,
            comic_id,
            comic_id,
        )
        .fetch_one(executor)
        .await
    }

    /// Returns first, last, previous, and next comic IDs that are missing a storyline item,
    /// in a single query against the `v_comics_missing_storyline` view.
    ///
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn missing_storyline_navigation<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
    ) -> sqlx::Result<NavigationResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            NavigationResult,
            r#"
                SELECT
                    MIN(`id`) AS "first?: u16",
                    MAX(`id`) AS "last?: u16",
                    MAX(CASE WHEN `id` < ? THEN `id` END) AS "previous?: u16",
                    MIN(CASE WHEN `id` > ? THEN `id` END) AS "next?: u16"
                FROM `v_comics_missing_storyline`
            "#,
            comic_id,
            comic_id,
        )
        .fetch_one(executor)
        .await
    }

    /// Returns first, last, previous, and next comic IDs that are missing a title,
    /// in a single query against the `v_comics_missing_title` view.
    ///
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn missing_title_navigation<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
    ) -> sqlx::Result<NavigationResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            NavigationResult,
            r#"
                SELECT
                    MIN(`id`) AS "first?: u16",
                    MAX(`id`) AS "last?: u16",
                    MAX(CASE WHEN `id` < ? THEN `id` END) AS "previous?: u16",
                    MIN(CASE WHEN `id` > ? THEN `id` END) AS "next?: u16"
                FROM `v_comics_missing_title`
            "#,
            comic_id,
            comic_id,
        )
        .fetch_one(executor)
        .await
    }

    /// Returns first, last, previous, and next comic IDs that are missing a tagline,
    /// in a single query against the `v_comics_missing_tagline` view.
    ///
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn missing_tagline_navigation<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
    ) -> sqlx::Result<NavigationResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            NavigationResult,
            r#"
                SELECT
                    MIN(`id`) AS "first?: u16",
                    MAX(`id`) AS "last?: u16",
                    MAX(CASE WHEN `id` < ? THEN `id` END) AS "previous?: u16",
                    MIN(CASE WHEN `id` > ? THEN `id` END) AS "next?: u16"
                FROM `v_comics_missing_tagline`
            "#,
            comic_id,
            comic_id,
        )
        .fetch_one(executor)
        .await
    }

    // ---

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn tagline_by_id<'e, 'c: 'e, E>(executor: E, id: u16) -> sqlx::Result<Option<String>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT `tagline` FROM `Comic` WHERE `id` = ?
            "#,
            id
        )
        .fetch_optional(executor)
        .await
        .map(Option::flatten)
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn update_tagline_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        tagline: &str,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                UPDATE `Comic`
                SET `tagline` = ?
                WHERE
                    `id` = ?
            "#,
            tagline,
            id
        )
        .execute(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn title_by_id<'e, 'c: 'e, E>(executor: E, id: u16) -> sqlx::Result<Option<String>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT `title` FROM `Comic` WHERE `id` = ?
            "#,
            id
        )
        .fetch_optional(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn update_title_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        title: &str,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                UPDATE `Comic`
                SET `title` = ?
                WHERE
                    `id` = ?
            "#,
            title,
            id
        )
        .execute(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn needs_updating_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
    ) -> sqlx::Result<(bool, bool, Option<NaiveDateTime>, bool)>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        if let Some(needs) = sqlx::query_as!(
            NeedsQuery,
            r#"
                SELECT
                    `title`,
                    `image_type`,
                    `publish_date`,
                    `hidden`
                FROM `Comic`
                WHERE `id` = ?
            "#,
            id
        )
        .fetch_optional(executor)
        .await?
        {
            Ok((
                needs.title.is_empty(),
                needs.image_type == 0,
                needs.publish_date,
                needs.hidden != 0,
            ))
        } else {
            Ok((true, true, None, false))
        }
    }

    /// Clears the `hidden` flag on a comic, e.g. once the background comic updater confirms
    /// via the front page that a previously-hidden advance comic has now been published.
    ///
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn unhide_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                UPDATE `Comic`
                SET `hidden` = 0
                WHERE `id` = ?
            "#,
            id,
        )
        .execute(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn insert_or_update_title_imagetype_and_publish_date_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        title: &str,
        image_type: i32,
        publish_date: NaiveDateTime,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                INSERT INTO `Comic`
                    (`id`, `title`, `image_type`, `publish_date`, `is_accurate_publish_date`, `hidden`)
                VALUES
                    (?, ?, ?, ?, 0, 0)
                ON DUPLICATE KEY UPDATE
                    `title` = ?,
                    `image_type` = ?,
                    `publish_date` = ?,
                    `hidden` = 0
            "#,
            id,
            title,
            image_type,
            publish_date,
            title,
            image_type,
            publish_date,
        )
        .execute(executor)
        .await
    }

    /// Inserts a hidden advance comic, or updates it in place if it's still hidden.
    ///
    /// Callers must check the comic isn't already a published (non-hidden) comic before
    /// calling this, e.g. via [`Comic::by_id`].
    ///
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[expect(clippy::too_many_arguments)]
    #[tracing::instrument(skip(executor))]
    pub async fn insert_advance_comic<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        title: &str,
        tagline: Option<&str>,
        publish_date: Option<NaiveDateTime>,
        is_accurate_publish_date: bool,
        is_guest_comic: bool,
        is_non_canon: bool,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                INSERT INTO `Comic`
                    (`id`, `title`, `tagline`, `publish_date`, `is_accurate_publish_date`, `is_guest_comic`, `is_non_canon`, `hidden`)
                VALUES
                    (?, ?, ?, ?, ?, ?, ?, 1)
                ON DUPLICATE KEY UPDATE
                    `title` = ?,
                    `tagline` = ?,
                    `publish_date` = ?,
                    `is_accurate_publish_date` = ?,
                    `is_guest_comic` = ?,
                    `is_non_canon` = ?,
                    `hidden` = 1
            "#,
            id,
            title,
            tagline,
            publish_date,
            is_accurate_publish_date,
            is_guest_comic,
            is_non_canon,
            title,
            tagline,
            publish_date,
            is_accurate_publish_date,
            is_guest_comic,
            is_non_canon,
        )
        .execute(executor)
        .await
    }

    /// # Errors
    ///
    /// Returns a database error if the query fails.
    #[tracing::instrument(skip(executor))]
    pub async fn update_image_type_and_publish_date_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        image_type: i32,
        publish_date: NaiveDateTime,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                UPDATE `Comic`
                SET
                    `image_type` = ?,
                    `publish_date` = ?,
                    `is_accurate_publish_date` = 0
                WHERE `id` = ?
            "#,
            image_type,
            publish_date,
            id
        )
        .execute(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
struct FirstLast {
    first: Option<u16>,
    last: Option<u16>,
}

#[derive(Copy, Clone, Debug, sqlx::FromRow)]
pub struct NavigationResult {
    pub first: Option<u16>,
    pub last: Option<u16>,
    pub previous: Option<u16>,
    pub next: Option<u16>,
}

#[derive(Debug)]
pub struct ComicWithNavigationAndNews {
    pub id: u16,
    pub image_type: i32,
    pub is_guest_comic: u8,
    pub is_non_canon: u8,
    pub has_no_cast: u8,
    pub has_no_location: u8,
    pub has_no_storyline: u8,
    pub has_no_title: u8,
    pub has_no_tagline: u8,
    pub title: String,
    pub tagline: Option<String>,
    pub publish_date: Option<NaiveDateTime>,
    pub is_accurate_publish_date: u8,
    pub prev_id: Option<u16>,
    pub next_id: Option<u16>,
    pub news: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct NeedsQuery {
    title: String,
    image_type: i32,
    publish_date: Option<NaiveDateTime>,
    hidden: u8,
}
