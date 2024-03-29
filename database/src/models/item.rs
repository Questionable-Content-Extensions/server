use futures::TryStreamExt;

use std::collections::{BTreeMap, HashSet};

use crate::models::{ComicId, ItemId, ItemType};

#[derive(Debug)]
pub struct Item {
    pub id: u16,
    pub short_name: String,
    pub name: String,
    pub r#type: String,
    pub color_blue: u8,
    pub color_green: u8,
    pub color_red: u8,
    pub primary_image: Option<u32>,
}

impl Item {
    #[tracing::instrument(skip(executor))]
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
                SELECT `i`.*
                FROM `Item` `i`
                JOIN `Occurrence` `o` ON `o`.`item_id` = `i`.`id`
                WHERE `o`.`comic_id` = ?
            "#,
            comic_id,
        )
        .fetch(executor)
        .map_ok(|i| (i.id, i))
        .try_collect()
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn occurrences_in_comic_by_id<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
    ) -> sqlx::Result<HashSet<u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT `i`.id
                FROM `Item` `i`
                JOIN `Occurrence` `o` ON `o`.`item_id` = `i`.`id`
                WHERE `o`.`comic_id` = ?
            "#,
            comic_id,
        )
        .fetch(executor)
        .map_ok(|i| i)
        .try_collect()
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn all<'e, 'c: 'e, E>(executor: E) -> sqlx::Result<Vec<Self>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            Self,
            r#"
                SELECT *
                FROM `Item`
            "#,
        )
        .fetch_all(executor)
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn all_mapped_by_id<'e, 'c: 'e, E>(executor: E) -> sqlx::Result<BTreeMap<u16, Self>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            Self,
            r#"
                SELECT *
                FROM `Item`
            "#,
        )
        .fetch(executor)
        .map_ok(|i| (i.id, i))
        .try_collect()
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn by_id<'e, 'c: 'e, E>(executor: E, id: u16) -> sqlx::Result<Option<Self>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            Self,
            r#"
                SELECT * FROM `Item` WHERE `id` = ?
            "#,
            id
        )
        .fetch_optional(executor)
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn create<'e, 'c: 'e, E>(
        executor: E,
        name: &str,
        short_name: &str,
        r#type: ItemType,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                INSERT INTO `Item`
                    (`name`, `short_name`, `type`)
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

    #[tracing::instrument(skip(executor))]
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
                    `item_id` AS `id`,
                    MIN(`comic_id`) AS `first`,
                    MAX(`comic_id`) AS `last`,
                    COUNT(`comic_id`) AS `count`
                FROM `Occurrence`
                WHERE `item_id` = ?
                GROUP by `item_id`
            "#,
            id
        )
        .fetch_one(executor)
        .await
    }

    #[tracing::instrument(skip(executor))]
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
                    `i`.`id`,
                    MIN(`c`.`id`) as `first`,
                    MAX(`c`.`id`) as `last`,
                    COUNT(`c`.`id`) as `count`
                FROM `Item` `i`
                JOIN `Occurrence` `o` ON `o`.`item_id` = `i`.`id`
                JOIN `Comic` `c` ON `c`.`id` = `o`.`comic_id`
                    AND (? is NULL OR `c`.`is_guest_comic` = ?)
                    AND (? is NULL OR `c`.`is_non_canon` = ?)
                GROUP by `i`.`id`
                ORDER BY `count` DESC
            "#,
            include_guest_comics,
            include_guest_comics,
            include_non_canon_comics,
            include_non_canon_comics,
        )
        .fetch_all(executor)
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn previous_apperances_by_comic_id_mapped_by_id<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
        include_guest_comics: Option<bool>,
        include_non_canon_comics: Option<bool>,
    ) -> sqlx::Result<PreviousAppearances>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        let mut order = Vec::new();
        let map = sqlx::query_as!(
            PrevNext,
            r#"
                SELECT
                    `i`.`id` as `id`,
                    MAX(`c`.`id`) as `comic`
                FROM `Item` `i`
                JOIN `Occurrence` `o` ON `o`.`item_id` = `i`.`id`
                JOIN `Comic` `c` ON `c`.`id` = `o`.`comic_id`
                WHERE `c`.`id` < ?
                    AND (? is NULL OR `c`.`is_guest_comic` = ?)
                    AND (? is NULL OR `c`.`is_non_canon` = ?)
                GROUP BY `i`.`id`
                ORDER BY `comic` DESC
            "#,
            comic_id,
            include_guest_comics,
            include_guest_comics,
            include_non_canon_comics,
            include_non_canon_comics,
        )
        .fetch(executor)
        .try_filter_map(|pn| {
            if pn.comic.is_some() {
                order.push(ItemId::from(pn.id));
            }
            async move {
                Ok(pn
                    .comic
                    .map(|comic| (ItemId::from(pn.id), ComicId::from(comic))))
            }
        })
        .try_collect()
        .await?;

        Ok(PreviousAppearances {
            appearances: map,
            order,
        })
    }

    #[tracing::instrument(skip(executor))]
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
                SELECT
                    `i`.`id` as `id`,
                    MIN(`c`.`id`) as `comic`
                FROM `Item` `i`
                JOIN `Occurrence` `o` ON `o`.`item_id` = `i`.`id`
                JOIN `Comic` `c` ON `c`.`id` = `o`.`comic_id`
                WHERE `c`.`id` > ?
                    AND (? is NULL OR `c`.`is_guest_comic` = ?)
                    AND (? is NULL OR `c`.`is_non_canon` = ?)
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
                Ok(Some((pn.id, comic)))
            } else {
                Ok(None)
            }
        })
        .try_collect()
        .await
    }

    // ---

    #[tracing::instrument(skip(executor))]
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
                    `i`.`id`,
                    MIN(`c`.`id`) as `first`,
                    MAX(`c`.`id`) as `last`,
                    COUNT(`c`.`id`) as `count`
                FROM `Item` `i`
                JOIN `Occurrence` `o` ON `o`.`item_id` = `i`.`id`
                JOIN `Comic` `c` ON `c`.`id` = `o`.`comic_id`
                    AND (? is NULL OR `c`.`is_guest_comic` = ?)
                    AND (? is NULL OR `c`.`is_non_canon` = ?)
                WHERE `i`.`id` IN (
                    SELECT `item_id` FROM `Occurrence` WHERE `comic_id` = ?
                )
                GROUP by `i`.`id`
                ORDER BY `count` DESC
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

    #[tracing::instrument(skip(executor))]
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
                SELECT
                    `i`.`id` as `id`,
                    MAX(`c`.`id`) as `comic`
                FROM `Item` `i`
                JOIN `Occurrence` `o` ON `o`.`item_id` = `i`.`id`
                JOIN `Comic` `c` ON `c`.`id` = `o`.`comic_id`
                WHERE `c`.`id` < ?
                    AND `i`.`id` IN (
                        SELECT `item_id` FROM `Occurrence` WHERE `comic_id` = ?
                    )
                    AND (? is NULL OR `c`.`is_guest_comic` = ?)
                    AND (? is NULL OR `c`.`is_non_canon` = ?)
                GROUP BY `i`.`id`
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
                Ok(Some((pn.id, comic)))
            } else {
                Ok(None)
            }
        })
        .try_collect()
        .await
    }

    #[tracing::instrument(skip(executor))]
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
                SELECT
                    `i`.`id` as `id`,
                    MIN(`c`.`id`) as `comic`
                FROM `Item` `i`
                JOIN `Occurrence` `o` ON `o`.`item_id` = `i`.`id`
                JOIN `Comic` `c` ON `c`.`id` = `o`.`comic_id`
                WHERE `c`.`id` > ?
                    AND `i`.`id` IN (
                        SELECT `item_id` FROM `Occurrence` WHERE `comic_id` = ?
                    )
                    AND (? is NULL OR `c`.`is_guest_comic` = ?)
                    AND (? is NULL OR `c`.`is_non_canon` = ?)
                GROUP BY `i`.`id`
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
                Ok(Some((pn.id, comic)))
            } else {
                Ok(None)
            }
        })
        .try_collect()
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn image_count_by_id<'e, 'c: 'e, E>(executor: E, id: u16) -> sqlx::Result<i64>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT COUNT(*) FROM `ItemImage`
                WHERE `item_id` = ?
            "#,
            id
        )
        .fetch_one(executor)
        .await
    }

    #[tracing::instrument(skip(executor))]
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
                    `id`,
                    `crc32c_hash`
                FROM `ItemImage`
                WHERE `item_id` = ?
                ORDER BY `id`
            "#,
            id
        )
        .fetch_all(executor)
        .await
    }

    #[tracing::instrument(skip(executor, map))]
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
                    `id`,
                    `crc32c_hash`
                FROM `ItemImage`
                WHERE `item_id` = ?
                ORDER BY `id`
            "#,
            id
        )
        .fetch(executor)
        .map_ok(map)
        .try_collect()
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn image_by_image_id<'e, 'c: 'e, E>(
        executor: E,
        image_id: u32,
    ) -> sqlx::Result<Option<Vec<u8>>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT `image`
                FROM `ItemImage`
                WHERE `id` = ?
            "#,
            image_id
        )
        .fetch_optional(executor)
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn item_id_by_image_id<'e, 'c: 'e, E>(
        executor: E,
        image_id: u32,
    ) -> sqlx::Result<Option<u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT `item_id`
                FROM `ItemImage`
                WHERE `id` = ?
            "#,
            image_id
        )
        .fetch_optional(executor)
        .await
    }

    #[tracing::instrument(skip(executor,image), fields(image.size = image.len()))]
    pub async fn create_image<'e, 'c: 'e, E>(
        executor: E,
        item_id: u16,
        image: Vec<u8>,
        crc32c_hash: u32,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                INSERT INTO `ItemImage`
                    (`item_id`, `image`, `crc32c_hash`)
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

    #[tracing::instrument(skip(executor))]
    pub async fn delete_image<'e, 'c: 'e, E>(
        executor: E,
        image_id: u32,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                DELETE FROM `ItemImage`
                WHERE `id` = ?
            "#,
            image_id
        )
        .execute(executor)
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn set_primary_image<'e, 'c: 'e, E>(
        executor: E,
        item_id: u16,
        image_id: u32,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                UPDATE `Item`
                SET `primary_image` = ?
                WHERE `id` = ?
            "#,
            image_id,
            item_id
        )
        .execute(executor)
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn update_name_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        name: &str,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                UPDATE `Item`
                SET `name` = ?
                WHERE `id` = ?
            "#,
            name,
            id
        )
        .execute(executor)
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn update_short_name_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        short_name: &str,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                UPDATE `Item`
                SET `short_name` = ?
                WHERE `id` = ?
            "#,
            short_name,
            id
        )
        .execute(executor)
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn update_color_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        red: u8,
        green: u8,
        blue: u8,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                UPDATE `Item`
                SET
                    `color_red` = ?,
                    `color_green` = ?,
                    `color_blue` = ?
                WHERE `id` = ?
            "#,
            red,
            green,
            blue,
            id
        )
        .execute(executor)
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn update_type_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: u16,
        r#type: ItemType,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                UPDATE `Item`
                SET `type` = ?
                WHERE `id` = ?
            "#,
            r#type,
            id
        )
        .execute(executor)
        .await
    }

    // Until v1 of API is gone
    #[allow(deprecated)]
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
                    `i2`.`id`,
                    `i2`.`short_name`,
                    `i2`.`name`,
                    `i2`.`type`,
                    `i2`.`color_blue`,
                    `i2`.`color_green`,
                    `i2`.`color_red`,
                    COUNT(`i2`.`id`) as `count`
                FROM `Item` `i`
                JOIN `Occurrence` `o` ON `i`.`id` = `o`.`item_id`
                JOIN `Occurrence` `o2` ON `o`.`comic_id` = `o2`.`comic_id`
                JOIN Item `i2` ON `o2`.`item_id` = `i2`.`id`
                WHERE `i`.`id` = ?
                    AND `i2`.`id` <> `i`.`id`
                    AND `i2`.`type` = ?
                GROUP BY `i2`.`id`
                ORDER BY `count` DESC
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
    pub id: u32,
    pub crc32c_hash: u32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ItemFirstLastCount {
    pub id: u16,
    pub first: Option<u16>,
    pub last: Option<u16>,
    pub count: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct PrevNext {
    id: u16,
    comic: Option<u16>,
}

pub struct PreviousAppearances {
    pub appearances: BTreeMap<ItemId, ComicId>,
    pub order: Vec<ItemId>,
}

// Can't deprecate these fields because `sqlx::FromRow` causes them to be
// used and trigger warnings.
#[derive(Debug, sqlx::FromRow)]
pub struct RelatedItem {
    pub id: u16,
    //#[deprecated(note = "Only needed in v1 now")]
    pub short_name: String,
    //#[deprecated(note = "Only needed in v1 now")]
    pub name: String,
    //#[deprecated(note = "Only needed in v1 now")]
    pub r#type: String,
    //#[deprecated(note = "Only needed in v1 now")]
    pub color_red: u8,
    //#[deprecated(note = "Only needed in v1 now")]
    pub color_green: u8,
    //#[deprecated(note = "Only needed in v1 now")]
    pub color_blue: u8,
    pub count: i64,
}
