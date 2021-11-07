use chrono::{NaiveDateTime, Utc};
use futures::TryStreamExt;
use log::info;

const PAGE_SIZE: u16 = 10;

#[derive(Debug)]
pub struct LogEntry {
    pub id: u32,
    pub user_token: String,
    pub date_time: NaiveDateTime,
    pub action: String,
}

impl LogEntry {
    pub async fn count<'e, 'c: 'e, E>(executor: E) -> sqlx::Result<i64>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT COUNT(*) FROM `LogEntry`
            "#,
        )
        .fetch_one(executor)
        .await
    }

    pub async fn by_page_number_with_mapping<'e, 'c: 'e, E, T, F>(
        executor: E,
        page: u16,
        map: F,
    ) -> sqlx::Result<Vec<T>>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
        F: FnMut(LogListEntry) -> T,
    {
        let start_entry = (page.saturating_sub(1)) * PAGE_SIZE;

        sqlx::query_as!(
            LogListEntry,
            r#"
                SELECT
                    `t`.`identifier`,
                    `l`.`date_time`,
                    `l`.`action`
                FROM `LogEntry` `l`
                JOIN `Token` `t` ON `t`.`id` = `l`.`user_token`
                ORDER BY `date_time` DESC
                LIMIT ?, ?
            "#,
            start_entry,
            PAGE_SIZE,
        )
        .fetch(executor)
        .map_ok(map)
        .try_collect()
        .await
    }

    pub async fn create<'e, 'c: 'e, E>(
        executor: E,
        token: &str,
        date_time: NaiveDateTime,
        action: &str,
    ) -> sqlx::Result<crate::DatabaseResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                INSERT INTO `LogEntry`
                    (`user_token`, `date_time`, `action`)
                VALUES
                    (?, ?, ?)
            "#,
            token,
            date_time,
            action,
        )
        .execute(executor)
        .await
    }

    pub async fn log_action<'e, 'c: 'e, E>(
        executor: E,
        token: impl AsRef<str>,
        action: impl AsRef<str>,
    ) -> sqlx::Result<()>
    where
        E: 'e + sqlx::Executor<'c, Database = sqlx::MySql>,
    {
        let token = token.as_ref();
        let action = action.as_ref();

        Self::create(executor, token, Utc::now().naive_utc(), action).await?;

        info!("{}", action);

        Ok(())
    }
}

pub struct LogListEntry {
    pub identifier: String,
    pub date_time: NaiveDateTime,
    pub action: String,
}
