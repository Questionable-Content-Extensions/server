use chrono::NaiveDate;
use log::debug;

#[derive(Debug)]
pub struct News {
    pub comic: i16,
    pub lastUpdated: NaiveDate,
    pub news: String,
    pub updateFactor: f64,
    pub isLocked: i8,
}

impl News {
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
				SELECT * FROM `news`
				WHERE `comic` = ?
			"#,
            comic_id
        )
        .fetch_optional(executor)
        .await
    }

    pub async fn update_last_updated_by_comic_id<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
        update_factor: f64,
        last_updated: NaiveDate,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                UPDATE `news`
                SET
                    updateFactor = ?,
                    lastUpdated = ?
                WHERE
                    comic = ?
            "#,
            update_factor,
            last_updated,
            comic_id,
        )
        .execute(executor)
        .await
    }

    pub async fn update_by_comic_id<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
        news: &str,
        update_factor: f64,
        last_updated: NaiveDate,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                UPDATE `news`
                SET
                    news = ?,
                    updateFactor = ?,
                    lastUpdated = ?
                WHERE
                    comic = ?
            "#,
            news,
            update_factor,
            last_updated,
            comic_id,
        )
        .execute(executor)
        .await
    }

    pub async fn create_for_comic_id<'e, 'c: 'e, E>(
        executor: E,
        comic_id: u16,
        news: &str,
        update_factor: f64,
        last_updated: NaiveDate,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                INSERT INTO `news`
                    (news, updateFactor, lastUpdated, comic)
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
        let days_since_update = (chrono::Utc::today().naive_utc() - self.lastUpdated).num_days();
        let update_factor_days = (31.0 * self.updateFactor) as i64;
        let locked = self.isLocked != 0;

        debug!(
            "News outdated info: id:{} updated:{} factor:{} locked:{} | ({} > {} = {})",
            self.comic,
            self.lastUpdated,
            self.updateFactor,
            locked,
            days_since_update,
            update_factor_days,
            days_since_update > update_factor_days
        );

        !locked && self.updateFactor < 12.0 && days_since_update > update_factor_days
    }
}
