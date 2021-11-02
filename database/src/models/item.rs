use futures::TryStreamExt;

use std::collections::BTreeMap;

use crate::models::ItemType;

#[derive(Debug)]
pub struct Item {
    pub id: i16,
    pub shortName: String,
    pub name: String,
    pub r#type: String,
    pub Color_Blue: u8,
    pub Color_Green: u8,
    pub Color_Red: u8,
}

impl Item {
    pub async fn occurrences_in_comic_mapped_by_id<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
    ) -> sqlx::Result<BTreeMap<u16, Self>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            Self,
            r#"
                SELECT i.*
                FROM items i
                JOIN occurences o ON o.items_id = i.id
                WHERE o.comic_id = ?
            "#,
            comic_id,
        )
        .fetch(executor)
        .map_ok(|i| (i.id as u16, i))
        .try_collect()
        .await
    }

    pub async fn all<'e, 'c: 'e, E>(executor: E) -> sqlx::Result<Vec<Self>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            Self,
            r#"
                SELECT *
                FROM items
            "#,
        )
        .fetch_all(executor)
        .await
    }

    pub async fn all_mapped_by_id<'e, 'c: 'e, E>(executor: E) -> sqlx::Result<BTreeMap<u16, Self>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            Self,
            r#"
                SELECT *
                FROM items
            "#,
        )
        .fetch(executor)
        .map_ok(|i| (i.id as u16, i))
        .try_collect()
        .await
    }

    pub async fn by_id<'e, 'c: 'e, E>(executor: E, id: u16) -> sqlx::Result<Option<Self>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            Self,
            r#"
                SELECT * FROM `items` WHERE id = ?
            "#,
            id
        )
        .fetch_optional(executor)
        .await
    }

    pub async fn create<'e, 'c: 'e, E>(
        executor: E,
        name: &str,
        short_name: &str,
        r#type: ItemType,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                INSERT INTO `items`
                    (name, shortName, type)
                VALUES
                    (?, ?, ?)
            "#,
            name,
            short_name,
            r#type,
        )
        .execute(executor)
        .await
    }

    pub async fn first_and_last_apperance_and_count_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
    ) -> sqlx::Result<ItemFirstLastCount>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            ItemFirstLastCount,
            r#"
                SELECT
                    items_id AS id,
                    MIN(comic_id) AS first,
                    MAX(comic_id) AS last,
                    COUNT(comic_id) AS count
                FROM `occurences`
                WHERE `items_id` = ?
            "#,
            id
        )
        .fetch_one(executor)
        .await
    }

    pub async fn first_and_last_apperances_and_count<'e, 'c: 'e, E>(
        executor: E,
        include_guest_comics: Option<bool>,
        include_non_canon_comics: Option<bool>,
    ) -> sqlx::Result<Vec<ItemFirstLastCount>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            ItemFirstLastCount,
            r#"
                SELECT
                    i.id,
                    MIN(c.id) as first,
                    MAX(c.id) as last,
                    COUNT(c.id) as count
                FROM items i
                JOIN occurences o ON o.items_id = i.id
                JOIN comic c ON c.id = o.comic_id
                    AND (? is NULL OR c.isGuestComic = ?)
                    AND (? is NULL OR c.isNonCanon = ?)
                GROUP by i.id
                ORDER BY count DESC
            "#,
            include_guest_comics,
            include_guest_comics,
            include_non_canon_comics,
            include_non_canon_comics,
        )
        .fetch_all(executor)
        .await
    }

    pub async fn previous_apperances_by_comic_id_mapped_by_id<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
        include_guest_comics: Option<bool>,
        include_non_canon_comics: Option<bool>,
    ) -> sqlx::Result<BTreeMap<u16, u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            PrevNext,
            r#"
                SELECT i.id as id, MAX(c.id) as comic
                FROM items i
                JOIN occurences o ON o.items_id = i.id
                JOIN comic c ON c.id = o.comic_id
                WHERE c.id < ?
                    AND (? is NULL OR c.isGuestComic = ?)
                    AND (? is NULL OR c.isNonCanon = ?)
                GROUP BY i.id
            "#,
            comic_id,
            include_guest_comics,
            include_guest_comics,
            include_non_canon_comics,
            include_non_canon_comics,
        )
        .fetch(executor)
        .try_filter_map(|pn| async move {
            if let Some(comic) = pn.comic {
                Ok(Some((pn.id as u16, comic as u16)))
            } else {
                Ok(None)
            }
        })
        .try_collect()
        .await
    }

    pub async fn next_apperances_by_comic_id_mapped_by_id<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
        include_guest_comics: Option<bool>,
        include_non_canon_comics: Option<bool>,
    ) -> sqlx::Result<BTreeMap<u16, u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            PrevNext,
            r#"
                SELECT i.id as id, MIN(c.id) as comic
                FROM items i
                JOIN occurences o ON o.items_id = i.id
                JOIN comic c ON c.id = o.comic_id
                WHERE c.id > ?
                    AND (? is NULL OR c.isGuestComic = ?)
                    AND (? is NULL OR c.isNonCanon = ?)
                GROUP BY i.id
            "#,
            comic_id,
            include_guest_comics,
            include_guest_comics,
            include_non_canon_comics,
            include_non_canon_comics,
        )
        .fetch(executor)
        .try_filter_map(|pn| async move {
            if let Some(comic) = pn.comic {
                Ok(Some((pn.id as u16, comic as u16)))
            } else {
                Ok(None)
            }
        })
        .try_collect()
        .await
    }

    // ---

    pub async fn first_and_last_apperances_and_count_of_items_in_comic_by_comic_id<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
        include_guest_comics: Option<bool>,
        include_non_canon_comics: Option<bool>,
    ) -> sqlx::Result<Vec<ItemFirstLastCount>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            ItemFirstLastCount,
            r#"
                SELECT
                    i.id,
                    MIN(c.id) as first,
                    MAX(c.id) as last,
                    COUNT(c.id) as count
                FROM items i
                JOIN occurences o ON o.items_id = i.id
                JOIN comic c ON c.id = o.comic_id
                    AND (? is NULL OR c.isGuestComic = ?)
                    AND (? is NULL OR c.isNonCanon = ?)
                WHERE i.id IN (
                    SELECT items_id FROM `occurences` WHERE comic_id = ?
                )
                GROUP by i.id
                ORDER BY count DESC
            "#,
            include_guest_comics,
            include_guest_comics,
            include_non_canon_comics,
            include_non_canon_comics,
            comic_id
        )
        .fetch_all(executor)
        .await
    }

    pub async fn previous_apperances_of_items_in_comic_by_comic_id<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
        include_guest_comics: Option<bool>,
        include_non_canon_comics: Option<bool>,
    ) -> sqlx::Result<BTreeMap<u16, u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            PrevNext,
            r#"
                SELECT i.id as id, MAX(c.id) as comic
                FROM items i
                JOIN occurences o ON o.items_id = i.id
                JOIN comic c ON c.id = o.comic_id
                WHERE c.id < ?
                    AND i.id IN (
                        SELECT items_id FROM `occurences` WHERE comic_id = ?
                    )
                    AND (? is NULL OR c.isGuestComic = ?)
                    AND (? is NULL OR c.isNonCanon = ?)
                GROUP BY i.id
            "#,
            comic_id,
            comic_id,
            include_guest_comics,
            include_guest_comics,
            include_non_canon_comics,
            include_non_canon_comics,
        )
        .fetch(executor)
        .try_filter_map(|pn| async move {
            if let Some(comic) = pn.comic {
                Ok(Some((pn.id as u16, comic as u16)))
            } else {
                Ok(None)
            }
        })
        .try_collect()
        .await
    }

    pub async fn next_apperances_of_items_in_comic_by_comic_id<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
        include_guest_comics: Option<bool>,
        include_non_canon_comics: Option<bool>,
    ) -> sqlx::Result<BTreeMap<u16, u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            PrevNext,
            r#"
                SELECT i.id as id, MIN(c.id) as comic
                FROM items i
                JOIN occurences o ON o.items_id = i.id
                JOIN comic c ON c.id = o.comic_id
                WHERE c.id > ?
                AND i.id IN (
                    SELECT items_id FROM `occurences` WHERE comic_id = ?
                )
                    AND (? is NULL OR c.isGuestComic = ?)
                    AND (? is NULL OR c.isNonCanon = ?)
                GROUP BY i.id
            "#,
            comic_id,
            comic_id,
            include_guest_comics,
            include_guest_comics,
            include_non_canon_comics,
            include_non_canon_comics,
        )
        .fetch(executor)
        .try_filter_map(|pn| async move {
            if let Some(comic) = pn.comic {
                Ok(Some((pn.id as u16, comic as u16)))
            } else {
                Ok(None)
            }
        })
        .try_collect()
        .await
    }

    pub async fn image_count_by_id<'e, 'c: 'e, E>(executor: E, id: u16) -> sqlx::Result<i64>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT COUNT(*) FROM `ItemImages`
                WHERE ItemId = ?
            "#,
            id
        )
        .fetch_one(executor)
        .await
    }

    pub async fn image_metadatas_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
    ) -> sqlx::Result<Vec<ItemImageMetadata>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            ItemImageMetadata,
            r#"
                SELECT
                    Id,
                    CRC32CHash
                FROM `ItemImages`
                WHERE ItemId = ?
            "#,
            id
        )
        .fetch_all(executor)
        .await
    }

    pub async fn image_metadatas_by_id_with_mapping<'e, 'c: 'e, E, T, F>(
        executor: E,
        id: u16,
        map: F,
    ) -> sqlx::Result<Vec<T>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
        F: FnMut(ItemImageMetadata) -> T,
    {
        sqlx::query_as!(
            ItemImageMetadata,
            r#"
                SELECT
                    Id,
                    CRC32CHash
                FROM `ItemImages`
                WHERE ItemId = ?
            "#,
            id
        )
        .fetch(executor)
        .map_ok(map)
        .try_collect()
        .await
    }

    pub async fn image_by_image_id<'e, 'c: 'e, E>(
        executor: E,
        image_id: i32,
    ) -> sqlx::Result<Option<Vec<u8>>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT
                    Image
                FROM `ItemImages`
                WHERE ItemId = ?
            "#,
            image_id
        )
        .fetch_optional(executor)
        .await
    }

    pub async fn create_image<'e, 'c: 'e, E>(
        executor: E,
        item_id: u16,
        image: Vec<u8>,
        crc32c_hash: u32,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                INSERT INTO `ItemImages`
                    (ItemId, Image, CRC32CHash)
                VALUES
                    (?, ?, ?)
            "#,
            item_id,
            image,
            crc32c_hash,
        )
        .execute(executor)
        .await
    }

    pub async fn update_name_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        name: &str,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                UPDATE `items`
                SET name = ?
                WHERE
                    id = ?
            "#,
            name,
            id
        )
        .execute(executor)
        .await
    }

    pub async fn update_short_name_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        short_name: &str,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                UPDATE `items`
                SET shortName = ?
                WHERE
                    id = ?
            "#,
            short_name,
            id
        )
        .execute(executor)
        .await
    }

    pub async fn update_color_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        red: u8,
        green: u8,
        blue: u8,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                UPDATE `items`
                SET
                    Color_Red = ?,
                    Color_Green = ?,
                    Color_Blue = ?
                WHERE
                    id = ?
            "#,
            red,
            green,
            blue,
            id
        )
        .execute(executor)
        .await
    }

    pub async fn related_items_by_id_and_type_with_mapping<'e, 'c: 'e, E, T, F>(
        executor: E,
        id: u16,
        r#type: ItemType,
        amount: i64,
        map: F,
    ) -> sqlx::Result<Vec<T>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
        F: FnMut(RelatedItem) -> T,
    {
        sqlx::query_as!(
            RelatedItem,
            r#"
                SELECT
                    i2.id,
                    i2.shortName as short_name,
                    i2.name,
                    i2.type,
                    i2.Color_Red as color_red,
                    i2.Color_Green as color_green,
                    i2.Color_Blue as color_blue,
                    COUNT(i2.id) as count
                FROM items i
                JOIN occurences o ON i.id = o.items_id
                JOIN occurences o2 ON o.comic_id = o2.comic_id
                JOIN items i2 ON o2.items_id = i2.id
                WHERE i.id = ?
                    AND i2.id <> i.id
                    AND i2.type = ?
                GROUP BY i2.id
                ORDER BY count DESC
                LIMIT ?
                "#,
            id,
            r#type,
            amount
        )
        .fetch(executor)
        .map_ok(map)
        .try_collect()
        .await
    }
}

pub struct ItemImageMetadata {
    pub Id: i32,
    pub CRC32CHash: u32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ItemFirstLastCount {
    pub id: i16,
    pub first: Option<i16>,
    pub last: Option<i16>,
    pub count: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct PrevNext {
    id: i16,
    comic: Option<i16>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct RelatedItem {
    pub id: i16,
    pub short_name: String,
    pub name: String,
    pub r#type: String,
    pub color_red: u8,
    pub color_green: u8,
    pub color_blue: u8,
    pub count: i64,
}
