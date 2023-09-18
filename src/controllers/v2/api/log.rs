use crate::models::v1::{LogEntry, Token};
use crate::util::ensure_is_authorized;
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::AuthDetails;
use database::models::LogEntry as DatabaseLogEntry;
use database::DbPool;
use serde::{Deserialize, Serialize};
use shared::token_permissions;
use ts_rs::TS;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(get)))
        .service(web::resource("comic").route(web::get().to(get_by_comic)))
        .service(web::resource("item").route(web::get().to(get_by_item)));
}

const PAGE_SIZE: u16 = 10;

#[tracing::instrument(skip(pool, auth), fields(permissions = ?auth.permissions))]
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

    let log_entry_count = DatabaseLogEntry::count(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let log_entries =
        DatabaseLogEntry::by_page_number_with_mapping(&mut *conn, query.page, LogEntry::from)
            .await
            .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(LogResponse {
        log_entries,
        page: query.page,
        log_entry_count: i32::try_from(log_entry_count).unwrap(),
        page_count: (log_entry_count as f64 / f64::from(PAGE_SIZE)).ceil() as u16,
    }))
}

#[tracing::instrument(skip(pool, auth), fields(permissions = ?auth.permissions))]
async fn get_by_comic(
    pool: web::Data<DbPool>,
    query: web::Query<LogByIdQuery>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
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

    Ok(HttpResponse::Ok().json(LogResponse {
        log_entries,
        page: query.page,
        log_entry_count: i32::try_from(log_entry_count).unwrap(),
        page_count: (log_entry_count as f64 / f64::from(PAGE_SIZE)).ceil() as u16,
    }))
}

#[tracing::instrument(skip(pool, auth), fields(permissions = ?auth.permissions))]
async fn get_by_item(
    pool: web::Data<DbPool>,
    query: web::Query<LogByIdQuery>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
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

    Ok(HttpResponse::Ok().json(LogResponse {
        log_entries,
        page: query.page,
        log_entry_count: i32::try_from(log_entry_count).unwrap(),
        page_count: (log_entry_count as f64 / f64::from(PAGE_SIZE)).ceil() as u16,
    }))
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
struct LogQuery {
    #[allow(unused)]
    token: Token,
    page: u16,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
struct LogByIdQuery {
    #[allow(unused)]
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
