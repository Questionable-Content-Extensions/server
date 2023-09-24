use crate::api::v1::models::LogEntry;
use crate::models::Token;
use crate::util::ensure_is_authorized;
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::AuthDetails;
use database::models::LogEntry as DatabaseLogEntry;
use database::DbPool;
use serde::{Deserialize, Serialize};
use shared::token_permissions;

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
        log_entry_count,
        page_count: (log_entry_count as f64 / f64::from(PAGE_SIZE)).ceil() as u16,
    }))
}

#[derive(Debug, Deserialize)]
struct LogQuery {
    #[allow(unused)]
    token: Token,
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
