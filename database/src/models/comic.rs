use chrono::{DateTime, NaiveDateTime, Utc};
use futures::TryStreamExt;

use crate::models::ItemType;

#[derive(Debug)]
pub struct Comic {
    pub id: i16,
    pub ImageType: i32,
    pub isGuestComic: i8,
    pub isNonCanon: i8,
    pub HasNoCast: u8,
    pub HasNoLocation: u8,
    pub HasNoStoryline: u8,
    pub HasNoTitle: u8,
    pub HasNoTagline: u8,
    pub title: String,
    pub tagline: Option<String>,
    pub publishDate: Option<NaiveDateTime>,
    pub isAccuratePublishDate: i8,
}

impl Comic {
    pub async fn count<'e, 'c: 'e, E>(executor: E) -> sqlx::Result<i64>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT COUNT(*) FROM `comic`
            "#,
        )
        .fetch_one(executor)
        .await
    }

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
        sqlx::query_as!(
            Self,
            r#"
                SELECT * FROM `comic`
                WHERE (? is NULL OR `isGuestComic` = ?)
                    AND (? is NULL OR `isNonCanon` = ?)
                ORDER BY id ASC
            "#,
            include_guest_comics,
            include_guest_comics,
            include_non_canon_comics,
            include_non_canon_comics
        )
        .fetch(executor)
        .map_ok(map)
        .try_collect()
        .await
    }

    pub async fn ensure_exists_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                INSERT IGNORE INTO `comic`
                    (id)
                VALUES
                    (?)
            "#,
            id,
        )
        .execute(executor)
        .await
    }

    pub async fn exists_by_id<'e, 'c: 'e, E>(executor: E, id: u16) -> sqlx::Result<bool>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT COUNT(1) FROM `comic`
                WHERE
                    id = ?
            "#,
            id,
        )
        .fetch_one(executor)
        .await
        .map(|c| c == 1)
    }

    pub async fn by_id<'e, 'c: 'e, E>(executor: E, id: u16) -> sqlx::Result<Option<Self>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            Self,
            r#"
                SELECT * FROM `comic`
                WHERE `id` = ?
            "#,
            id
        )
        .fetch_optional(executor)
        .await
    }

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
                SELECT id
                FROM `comic`
                WHERE id < ?
                    AND (? is NULL OR `isGuestComic` = ?)
                    AND (? is NULL OR `isNonCanon` = ?)
                ORDER BY id DESC
            "#,
            id,
            include_guest_comics,
            include_guest_comics,
            include_non_canon_comics,
            include_non_canon_comics,
        )
        .fetch_optional(executor)
        .await
        .map(|i| i.map(|i| i as u16))
    }

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
                SELECT id
                FROM `comic`
                WHERE id > ?
                    AND (? is NULL OR `isGuestComic` = ?)
                    AND (? is NULL OR `isNonCanon` = ?)
                ORDER BY id ASC
            "#,
            id,
            include_guest_comics,
            include_guest_comics,
            include_non_canon_comics,
            include_non_canon_comics,
        )
        .fetch_optional(executor)
        .await
        .map(|i| i.map(|i| i as u16))
    }

    pub async fn publish_date_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
    ) -> sqlx::Result<Option<NaiveDateTime>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT publishDate FROM `comic` WHERE id = ?
            "#,
            id
        )
        .fetch_one(executor)
        .await
    }

    pub async fn update_publish_date_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        publish_date: DateTime<Utc>,
        is_accurate_publish_date: bool,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                UPDATE `comic`
                SET
                    publishDate = ?,
                    isAccuratePublishDate = ?
                WHERE
                    id = ?
            "#,
            publish_date.naive_utc(),
            is_accurate_publish_date,
            id,
        )
        .execute(executor)
        .await
    }

    pub async fn update_is_guest_comic_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        is_guest_comic: bool,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                UPDATE `comic`
                SET
                    isGuestComic = ?
                WHERE
                    id = ?
            "#,
            is_guest_comic,
            id,
        )
        .execute(executor)
        .await
    }

    pub async fn update_is_non_canon_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        is_non_canon: bool,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                UPDATE `comic`
                SET
                    isNonCanon = ?
                WHERE
                    id = ?
            "#,
            is_non_canon,
            id,
        )
        .execute(executor)
        .await
    }

    pub async fn update_has_no_cast_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        has_no_cast: bool,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                UPDATE `comic`
                SET
                    HasNoCast = ?
                WHERE
                    id = ?
            "#,
            has_no_cast,
            id,
        )
        .execute(executor)
        .await
    }

    pub async fn update_has_no_location_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        has_no_location: bool,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                UPDATE `comic`
                SET
                    HasNoLocation = ?
                WHERE
                    id = ?
            "#,
            has_no_location,
            id,
        )
        .execute(executor)
        .await
    }

    pub async fn update_has_no_storyline_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        has_no_storyline: bool,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                UPDATE `comic`
                SET
                    HasNoStoryline = ?
                WHERE
                    id = ?
            "#,
            has_no_storyline,
            id,
        )
        .execute(executor)
        .await
    }

    pub async fn update_has_no_title_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        has_no_title: bool,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                UPDATE `comic`
                SET
                    HasNoTitle = ?
                WHERE
                    id = ?
            "#,
            has_no_title,
            id,
        )
        .execute(executor)
        .await
    }

    pub async fn update_has_no_tagline_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        has_no_tagline: bool,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                UPDATE `comic`
                SET
                    HasNoTagline = ?
                WHERE
                    id = ?
            "#,
            has_no_tagline,
            id,
        )
        .execute(executor)
        .await
    }

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
                    MIN(c.id) as first,
                    MAX(c.id) as last
                FROM comic c
                WHERE (c.tagline IS NULL or NULLIF(c.tagline, '') IS NULL)
                    AND NOT c.HasNoTagline
                    AND c.id > 3132
            "#
        )
        .fetch_optional(executor)
        .await
        .map(|ofl| {
            ofl.map_or((None, None), |fl| {
                (fl.first.map(|i| i as u16), fl.last.map(|i| i as u16))
            })
        })
    }

    pub async fn previous_missing_tagline_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
    ) -> sqlx::Result<Option<u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT c.id
                FROM comic c
                WHERE (c.tagline IS NULL OR NULLIF(c.tagline, '') IS NULL)
                    AND NOT c.HasNoTagline
                    AND c.id < ?
                    AND c.id > 3132
                ORDER BY c.id DESC
                LIMIT 1
            "#,
            id
        )
        .fetch_optional(executor)
        .await
        .map(|i| i.map(|i| i as u16))
    }

    pub async fn next_missing_tagline_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
    ) -> sqlx::Result<Option<u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT c.id
                FROM comic c
                WHERE (c.tagline IS NULL OR NULLIF(c.tagline, '') IS NULL)
                    AND NOT c.HasNoTagline
                    AND c.id > ?
                    AND c.id > 3132
                ORDER BY c.id ASC
                LIMIT 1
            "#,
            id
        )
        .fetch_optional(executor)
        .await
        .map(|i| i.map(|i| i as u16))
    }

    // ---

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
                    MIN(c.id) as first,
                    MAX(c.id) as last
                FROM comic c
                WHERE (c.title IS NULL or NULLIF(c.title, '') IS NULL)
                    AND NOT c.HasNoTitle
            "#
        )
        .fetch_optional(executor)
        .await
        .map(|ofl| {
            ofl.map_or((None, None), |fl| {
                (fl.first.map(|i| i as u16), fl.last.map(|i| i as u16))
            })
        })
    }

    pub async fn previous_missing_title_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
    ) -> sqlx::Result<Option<u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT c.id
                FROM comic c
                WHERE (c.title IS NULL OR NULLIF(c.title, '') IS NULL)
                    AND NOT c.HasNoTitle
                    AND c.id < ?
                ORDER BY c.id DESC
                LIMIT 1
            "#,
            id
        )
        .fetch_optional(executor)
        .await
        .map(|i| i.map(|i| i as u16))
    }

    pub async fn next_missing_title_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
    ) -> sqlx::Result<Option<u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT c.id
                FROM comic c
                WHERE (c.title IS NULL OR NULLIF(c.title, '') IS NULL)
                    AND NOT c.HasNoTitle
                    AND c.id > ?
                ORDER BY c.id ASC
                LIMIT 1
            "#,
            id
        )
        .fetch_optional(executor)
        .await
        .map(|i| i.map(|i| i as u16))
    }

    // ---

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
                SELECT c.id
                FROM comic c
                WHERE c.id NOT IN
                    (
                        SELECT o.comic_id
                        FROM occurences o
                        LEFT JOIN items i ON o.items_id = i.id
                        WHERE i.type = ?
                        AND o.comic_id = c.id
                        GROUP BY o.comic_id
                    )
                    AND (? <> 'cast' OR NOT c.HasNoCast)
                    AND (? <> 'location' OR NOT c.HasNoLocation)
                    AND (? <> 'storyline' OR NOT c.HasNoStoryline)
                ORDER BY c.id ASC
                LIMIT 1
            "#,
            r#type,
            r#type,
            r#type,
            r#type
        )
        .fetch_optional(executor)
        .await
        .map(|i| i.map(|i| i as u16))
    }

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
                SELECT c.id
                FROM comic c
                WHERE c.id NOT IN
                    (
                        SELECT o.comic_id
                        FROM occurences o
                        LEFT JOIN items i ON o.items_id = i.id
                        WHERE i.type = ?
                        AND o.comic_id = c.id
                        GROUP BY o.comic_id
                    )
                    AND c.id < ?
                    AND (? <> 'cast' OR NOT c.HasNoCast)
                    AND (? <> 'location' OR NOT c.HasNoLocation)
                    AND (? <> 'storyline' OR NOT c.HasNoStoryline)
                ORDER BY c.id DESC
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
        .map(|i| i.map(|i| i as u16))
    }

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
                SELECT c.id
                FROM comic c
                WHERE c.id NOT IN
                    (
                        SELECT o.comic_id
                        FROM occurences o
                        LEFT JOIN items i ON o.items_id = i.id
                        WHERE i.type = ?
                        AND o.comic_id = c.id
                        GROUP BY o.comic_id
                    )
                    AND c.id > ?
                    AND (? <> 'cast' OR NOT c.HasNoCast)
                    AND (? <> 'location' OR NOT c.HasNoLocation)
                    AND (? <> 'storyline' OR NOT c.HasNoStoryline)
                ORDER BY c.id ASC
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
        .map(|i| i.map(|i| i as u16))
    }

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
                SELECT c.id
                FROM comic c
                WHERE c.id NOT IN
                    (
                        SELECT o.comic_id
                        FROM occurences o
                        LEFT JOIN items i ON o.items_id = i.id
                        WHERE i.type = ?
                        AND o.comic_id = c.id
                        GROUP BY o.comic_id
                    )
                    AND (? <> 'cast' OR NOT c.HasNoCast)
                    AND (? <> 'location' OR NOT c.HasNoLocation)
                    AND (? <> 'storyline' OR NOT c.HasNoStoryline)
                ORDER BY c.id DESC
                LIMIT 1
            "#,
            r#type,
            r#type,
            r#type,
            r#type
        )
        .fetch_optional(executor)
        .await
        .map(|i| i.map(|i| i as u16))
    }

    pub async fn tagline_by_id<'e, 'c: 'e, E>(executor: E, id: u16) -> sqlx::Result<Option<String>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT tagline FROM `comic` WHERE id = ?
            "#,
            id
        )
        .fetch_optional(executor)
        .await
        .map(|o| o.flatten())
    }

    pub async fn update_tagline_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        tagline: &str,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                UPDATE `comic`
                SET tagline = ?
                WHERE
                    id = ?
            "#,
            tagline,
            id
        )
        .execute(executor)
        .await
    }

    pub async fn title_by_id<'e, 'c: 'e, E>(executor: E, id: u16) -> sqlx::Result<Option<String>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT title FROM `comic` WHERE id = ?
            "#,
            id
        )
        .fetch_optional(executor)
        .await
    }

    pub async fn update_title_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        title: &str,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                UPDATE `comic`
                SET title = ?
                WHERE
                    id = ?
            "#,
            title,
            id
        )
        .execute(executor)
        .await
    }

    pub async fn needs_updating_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
    ) -> sqlx::Result<(bool, bool)>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        if let Some(needs) = sqlx::query_as!(
            NeedsQuery,
            r#"
                SELECT title, ImageType FROM `comic`
                WHERE `id` = ?
            "#,
            id
        )
        .fetch_optional(executor)
        .await?
        {
            Ok((needs.title.is_empty(), needs.ImageType == 0))
        } else {
            Ok((true, true))
        }
    }

    pub async fn insert_or_update_title_and_imagetype_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        title: &str,
        image_type: i32,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                INSERT INTO `comic`
                    (id, title, imagetype)
                VALUES
                    (?, ?, ?)
                ON DUPLICATE KEY UPDATE
                    title = ?,
                    imagetype = ?
            "#,
            id,
            title,
            image_type,
            title,
            image_type,
        )
        .execute(executor)
        .await
    }

    pub async fn update_image_type_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        image_type: i32,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                UPDATE `comic`
                SET
                    ImageType = ?
                WHERE id = ?
            "#,
            image_type,
            id
        )
        .execute(executor)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
struct FirstLast {
    first: Option<i16>,
    last: Option<i16>,
}

#[derive(Debug, sqlx::FromRow)]
struct NeedsQuery {
    title: String,
    ImageType: i32,
}
