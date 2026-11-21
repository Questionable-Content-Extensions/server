use crate::api::v2::models::LogEntry;
use crate::util::ensure_is_authorized;
use actix_web::{HttpResponse, Result, error, web};
use actix_web_grants::authorities::AuthDetails;
use database::DbPool;
use database::models::LogEntry as DatabaseLogEntry;
use serde::{Deserialize, Serialize};
use shared::token_permissions;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(get)))
        .service(web::resource("comic").route(web::get().to(get_by_comic)))
        .service(web::resource("item").route(web::get().to(get_by_item)));
}

const PAGE_SIZE: u16 = 10;

#[tracing::instrument(skip(pool, auth), fields(permissions = ?auth.authorities))]
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
        page_count: u16::try_from(
            (log_entry_count + i64::from(PAGE_SIZE) - 1) / i64::from(PAGE_SIZE),
        )
        .expect("page count fits in u16"),
    }))
}

#[tracing::instrument(skip(pool, auth), fields(permissions = ?auth.authorities))]
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
        page_count: u16::try_from(
            (log_entry_count + i64::from(PAGE_SIZE) - 1) / i64::from(PAGE_SIZE),
        )
        .expect("page count fits in u16"),
    }))
}

#[tracing::instrument(skip(pool, auth), fields(permissions = ?auth.authorities))]
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
        page_count: u16::try_from(
            (log_entry_count + i64::from(PAGE_SIZE) - 1) / i64::from(PAGE_SIZE),
        )
        .expect("page count fits in u16"),
    }))
}

/// The token comes from the permission middleware, which also reads a legacy `token` query
/// parameter; serde ignores that parameter here.
#[derive(Debug, Deserialize)]
struct LogQuery {
    page: u16,
}

/// See [`LogQuery`] for where the token comes from.
#[derive(Debug, Deserialize)]
struct LogByIdQuery {
    id: u16,
    page: u16,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LogResponse {
    log_entries: Vec<LogEntry>,
    page: u16,
    page_count: u16,
    log_entry_count: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = "00000000-0000-0000-0000-000000000000";

    #[test]
    fn log_query_does_not_require_a_token() {
        let query = web::Query::<LogQuery>::from_query("page=3").unwrap();

        assert_eq!(query.page, 3);
    }

    #[test]
    fn log_query_accepts_a_legacy_token_parameter() {
        let query = web::Query::<LogQuery>::from_query(&format!("token={TOKEN}&page=3")).unwrap();

        assert_eq!(query.page, 3);
    }

    #[test]
    fn log_by_id_query_does_not_require_a_token() {
        let query = web::Query::<LogByIdQuery>::from_query("id=7&page=3").unwrap();

        assert_eq!((query.id, query.page), (7, 3));
    }

    #[test]
    fn log_by_id_query_accepts_a_legacy_token_parameter() {
        let query =
            web::Query::<LogByIdQuery>::from_query(&format!("token={TOKEN}&id=7&page=3")).unwrap();

        assert_eq!((query.id, query.page), (7, 3));
    }
}
