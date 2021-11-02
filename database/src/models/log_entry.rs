use chrono::{NaiveDateTime, Utc};
use futures::TryStreamExt;
use log::info;

const PAGE_SIZE: u16 = 10;

#[derive(Debug)]
pub struct LogEntry {
    pub id: i32,
    pub UserToken: String,
    pub DateTime: NaiveDateTime,
    pub Action: String,
}

impl LogEntry {
    pub async fn count<'e, 'c: 'e, E>(executor: E) -> sqlx::Result<i64>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM `log_entry`
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
        F: FnMut(Self) -> T,
    {
        let start_entry = (page.saturating_sub(1)) * PAGE_SIZE;

        sqlx::query_as!(
            Self,
            r#"
            SELECT * FROM `log_entry`
            ORDER BY `DateTime` DESC
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
                INSERT INTO `log_entry`
                    (UserToken, DateTime, Action)
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
