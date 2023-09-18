use chrono::{NaiveDate, Utc};
use tracing::debug;

#[derive(Debug)]
pub struct News {
    pub comic_id: u16,
    pub last_updated: NaiveDate,
    pub news: String,
    pub update_factor: f64,
    pub is_locked: u8,
}

impl News {
    #[tracing::instrument(skip(executor))]
    pub async fn by_comic_id<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
    ) -> sqlx::Result<Option<Self>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_as!(
            Self,
            r#"
				SELECT * FROM `News`
				WHERE `comic_id` = ?
			"#,
            comic_id
        )
        .fetch_optional(executor)
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn update_last_updated_by_comic_id<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
        update_factor: f64,
        last_updated: NaiveDate,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                UPDATE `News`
                SET
                    `update_factor` = ?,
                    `last_updated` = ?
                WHERE
                    `comic_id` = ?
            "#,
            update_factor,
            last_updated,
            comic_id,
        )
        .execute(executor)
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn update_by_comic_id<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
        news: &str,
        update_factor: f64,
        last_updated: NaiveDate,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                UPDATE `News`
                SET
                    `news` = ?,
                    `update_factor` = ?,
                    `last_updated` = ?
                WHERE
                    `comic_id` = ?
            "#,
            news,
            update_factor,
            last_updated,
            comic_id,
        )
        .execute(executor)
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn create_for_comic_id<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
        news: &str,
        update_factor: f64,
        last_updated: NaiveDate,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                INSERT INTO `News`
                    (`news`, `update_factor`, `last_updated`, `comic_id`)
                VALUES
                    (?, ?, ?, ?)
            "#,
            news,
            update_factor,
            last_updated,
            comic_id,
        )
        .execute(executor)
        .await
    }
}

impl News {
    pub fn is_outdated(&self) -> bool {
        let days_since_update = (Utc::now().date_naive() - self.last_updated).num_days();
        let update_factor_days = (31.0 * self.update_factor) as i64;
        let locked = self.is_locked != 0;

        debug!(
            "News outdated info: id:{} updated:{} factor:{} locked:{} | ({} > {} = {})",
            self.comic_id,
            self.last_updated,
            self.update_factor,
            locked,
            days_since_update,
            update_factor_days,
            days_since_update > update_factor_days
        );

        !locked && self.update_factor < 12.0 && days_since_update > update_factor_days
    }
}
