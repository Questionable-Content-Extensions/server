use crate::database::models::LogEntry as DatabaseLogEntry;
use crate::database::DbPool;
use crate::models::{token_permissions, LogEntry};
use crate::util::ensure_is_authorized;
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::AuthDetails;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(get)));
}

const PAGE_SIZE: u16 = 10;

async fn get(
    pool: web::Data<DbPool>,
    query: web::Query<LogQuery>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    ensure_is_authorized(&auth, token_permissions::HAS_VALID_TOKEN)
        .map_err(error::ErrorForbidden)?;

    let mut conn = pool
        .acquire()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let log_entry_count = sqlx::query_scalar!(
        r#"
            SELECT COUNT(*) FROM `log_entry`
        "#,
    )
    .fetch_one(&mut conn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    let start_entry = (query.page.saturating_sub(1)) * PAGE_SIZE;

    let log_entries: Vec<LogEntry> = sqlx::query_as!(
        DatabaseLogEntry,
        r#"
            SELECT * FROM `log_entry`
            ORDER BY `DateTime` DESC
            LIMIT ?, 10
        "#,
        start_entry
    )
    .fetch(&mut conn)
    .map_err(anyhow::Error::from)
    .and_then(|l| futures::future::ready(TryFrom::try_from(l)))
    .try_collect()
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(LogResponse {
        log_entries,
        page: query.page,
        log_entry_count,
        page_count: (log_entry_count as f64 / f64::from(PAGE_SIZE)).ceil() as u16,
    }))
}

#[derive(Debug, Deserialize)]
struct LogQuery {
    token: uuid::Uuid,
    page: u16,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LogResponse {
    log_entries: Vec<LogEntry>,
    page: u16,
    page_count: u16,
    log_entry_count: i64,
}
