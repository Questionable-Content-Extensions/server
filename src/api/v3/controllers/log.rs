use crate::api::v3::models::LogEntry;
use crate::models::Token;
use crate::util::ensure_is_authorized;
use actix_web::web::Json;
use actix_web::{Result, error, web};
use actix_web_grants::authorities::AuthDetails;
use api_macros::api_endpoint;
use database::DbPool;
use database::models::LogEntry as DatabaseLogEntry;
use serde::{Deserialize, Serialize};
use shared::token_permissions;
use ts_rs::TS;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get).service(get_by_comic).service(get_by_item);
}

const PAGE_SIZE: u16 = 10;

#[api_endpoint(method = "GET", path = "log/")]
#[tracing::instrument(skip(pool, auth), fields(permissions = ?auth.authorities))]
async fn get(
    pool: web::Data<DbPool>,
    query: web::Query<LogQuery>,
    auth: AuthDetails,
) -> Result<Json<LogResponse>> {
    ensure_is_authorized(&auth, token_permissions::HAS_VALID_TOKEN)
        .map_err(error::ErrorForbidden)?;

    let mut conn = pool
        .acquire()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let log_entry_count = DatabaseLogEntry::count(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let log_entries =
        DatabaseLogEntry::by_page_number_with_mapping(&mut *conn, query.page, LogEntry::from)
            .await
            .map_err(error::ErrorInternalServerError)?;

    Ok(Json(LogResponse {
        log_entries,
        page: query.page,
        log_entry_count: i32::try_from(log_entry_count).unwrap(),
        page_count: u16::try_from(
            (log_entry_count + i64::from(PAGE_SIZE) - 1) / i64::from(PAGE_SIZE),
        )
        .expect("page count fits in u16"),
    }))
}

#[api_endpoint(method = "GET", path = "log/comic")]
#[tracing::instrument(skip(pool, auth), fields(permissions = ?auth.authorities))]
async fn get_by_comic(
    pool: web::Data<DbPool>,
    query: web::Query<LogByIdQuery>,
    auth: AuthDetails,
) -> Result<Json<LogResponse>> {
    ensure_is_authorized(&auth, token_permissions::HAS_VALID_TOKEN)
        .map_err(error::ErrorForbidden)?;

    let mut conn = pool
        .acquire()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let log_entry_count = DatabaseLogEntry::count_including_comic(&mut *conn, query.id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let log_entries = DatabaseLogEntry::including_comic_by_page_number_with_mapping(
        &mut *conn,
        query.id,
        query.page,
        LogEntry::from,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(Json(LogResponse {
        log_entries,
        page: query.page,
        log_entry_count: i32::try_from(log_entry_count).unwrap(),
        page_count: u16::try_from(
            (log_entry_count + i64::from(PAGE_SIZE) - 1) / i64::from(PAGE_SIZE),
        )
        .expect("page count fits in u16"),
    }))
}

#[api_endpoint(method = "GET", path = "log/item")]
#[tracing::instrument(skip(pool, auth), fields(permissions = ?auth.authorities))]
async fn get_by_item(
    pool: web::Data<DbPool>,
    query: web::Query<LogByIdQuery>,
    auth: AuthDetails,
) -> Result<Json<LogResponse>> {
    ensure_is_authorized(&auth, token_permissions::HAS_VALID_TOKEN)
        .map_err(error::ErrorForbidden)?;

    let mut conn = pool
        .acquire()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let log_entry_count = DatabaseLogEntry::count_including_item(&mut *conn, query.id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let log_entries = DatabaseLogEntry::including_item_by_page_number_with_mapping(
        &mut *conn,
        query.id,
        query.page,
        LogEntry::from,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(Json(LogResponse {
        log_entries,
        page: query.page,
        log_entry_count: i32::try_from(log_entry_count).unwrap(),
        page_count: u16::try_from(
            (log_entry_count + i64::from(PAGE_SIZE) - 1) / i64::from(PAGE_SIZE),
        )
        .expect("page count fits in u16"),
    }))
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
struct LogQuery {
    #[expect(unused)]
    token: Token,
    page: u16,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
struct LogByIdQuery {
    #[expect(unused)]
    token: Token,
    id: u16,
    page: u16,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
struct LogResponse {
    log_entries: Vec<LogEntry>,
    page: u16,
    page_count: u16,
    log_entry_count: i32,
}
