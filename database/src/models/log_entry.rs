use chrono::{NaiveDateTime, Utc};
use futures::TryStreamExt;
use tracing::info;

const PAGE_SIZE: u16 = 10;

#[derive(Debug)]
pub struct LogEntry {
    pub id: u32,
    pub user_token: String,
    pub date_time: NaiveDateTime,
    pub action: String,
    pub involved_comic: Option<u16>,
    pub involved_item: Option<u16>,
}

impl LogEntry {
    #[tracing::instrument(skip(executor))]
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

    #[tracing::instrument(skip(executor, map))]
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

    #[tracing::instrument(skip(executor))]
    pub async fn count_including_comic<'e, 'c: 'e, E>(executor: E, comic: u16) -> sqlx::Result<i64>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT COUNT(*) FROM `LogEntry`
                WHERE `comic_involved` = ?
            "#,
            comic,
        )
        .fetch_one(executor)
        .await
    }

    #[tracing::instrument(skip(executor, map))]
    pub async fn including_comic_by_page_number_with_mapping<'e, 'c: 'e, E, T, F>(
        executor: E,
        comic: u16,
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
                WHERE `l`.`comic_involved` = ?
                ORDER BY `date_time` DESC
                LIMIT ?, ?
            "#,
            comic,
            start_entry,
            PAGE_SIZE,
        )
        .fetch(executor)
        .map_ok(map)
        .try_collect()
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn count_including_item<'e, 'c: 'e, E>(executor: E, item: u16) -> sqlx::Result<i64>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query_scalar!(
            r#"
                SELECT COUNT(*) FROM `LogEntry`
                WHERE `item_involved` = ?
            "#,
            item,
        )
        .fetch_one(executor)
        .await
    }

    #[tracing::instrument(skip(executor, map))]
    pub async fn including_item_by_page_number_with_mapping<'e, 'c: 'e, E, T, F>(
        executor: E,
        item: u16,
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
                WHERE `l`.`item_involved` = ?
                ORDER BY `date_time` DESC
                LIMIT ?, ?
            "#,
            item,
            start_entry,
            PAGE_SIZE,
        )
        .fetch(executor)
        .map_ok(map)
        .try_collect()
        .await
    }

    #[tracing::instrument(skip(executor))]
    pub async fn create<'e, 'c: 'e, E>(
        executor: E,
        token: &str,
        date_time: NaiveDateTime,
        action: &str,
        involved_comic: Option<u16>,
        involved_item: Option<u16>,
    ) -> sqlx::Result<crate::DatabaseQueryResult>
    where
        E: 'e + sqlx::Executor<'c, Database = crate::DatabaseDriver>,
    {
        sqlx::query!(
            r#"
                INSERT INTO `LogEntry`
                    (`user_token`, `date_time`, `action`, `comic_involved`, `item_involved`)
                VALUES
                    (?, ?, ?, ?, ?)
            "#,
            token,
            date_time,
            action,
            involved_comic,
            involved_item,
        )
        .execute(executor)
        .await
    }

    #[tracing::instrument(skip(executor, token, action), fields(token = token.as_ref(), action = action.as_ref()))]
    pub async fn log_action<'e, 'c: 'e, E>(
        executor: E,
        token: impl AsRef<str>,
        action: impl AsRef<str>,
        involved_comic: Option<u16>,
        involved_item: Option<u16>,
    ) -> sqlx::Result<()>
    where
        E: 'e + sqlx::Executor<'c, Database = sqlx::MySql>,
    {
        let token = token.as_ref();
        let action = action.as_ref();

        Self::create(
            executor,
            token,
            Utc::now().naive_utc(),
            action,
            involved_comic,
            involved_item,
        )
        .await?;

        info!("{}", action);

        Ok(())
    }
}

pub struct LogListEntry {
    pub identifier: String,
    pub date_time: NaiveDateTime,
    pub action: String,
}
