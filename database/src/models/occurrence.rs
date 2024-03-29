#[derive(Debug)]
pub struct Occurrence {
    pub comic_id: i16,
    pub item_id: i16,
}

impl Occurrence {
    #[tracing::instrument(skip(executor))]
    pub async fn occurrence_by_item_id_and_comic_id<'e, 'c: 'e, E>(
        executor: E,
        item_id: u16,
        comic_id: u16,
    ) -> sqlx::Result<bool>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT COUNT(1) FROM `Occurrence`
                WHERE
                    `item_id` = ?
                AND
                    `comic_id` = ?
            "#,
            item_id,
            comic_id,
        )
        .fetch_one(executor)
        .await
        .map(|c| c == 1)
    }

    #[tracing::instrument(skip(executor))]
    pub async fn comic_id_occurrence_by_item_id<'e, 'c: 'e, E>(
        executor: E,
        item_id: u16,
    ) -> sqlx::Result<Vec<u16>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT `comic_id` FROM `Occurrence`
                WHERE
                    `item_id` = ?
            "#,
            item_id,
        )
        .fetch_all(executor)
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn create<'e, 'c: 'e, E>(
        executor: E,
        item_id: u16,
        comic_id: u16,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                INSERT INTO `Occurrence`
                    (`comic_id`, `item_id`)
                VALUES
                    (?, ?)
            "#,
            comic_id,
            item_id
        )
        .execute(executor)
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn delete<'e, 'c: 'e, E>(
        executor: E,
        item_id: u16,
        comic_id: u16,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                DELETE FROM `Occurrence`
                WHERE
                    `item_id` = ?
                AND
                    `comic_id` = ?
            "#,
            item_id,
            comic_id,
        )
        .execute(executor)
        .await
    }
}
