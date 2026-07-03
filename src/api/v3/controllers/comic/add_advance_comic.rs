use crate::api::v3::models::AdvanceComicListItem;
use crate::models::{ComicId, Token};
use crate::util::{ComicUpdaterTrigger, ensure_is_authorized};
use actix_web::web::Json;
use actix_web::{Result, error, web};
use actix_web_grants::authorities::AuthDetails;
use api_macros::api_endpoint;
use chrono::{DateTime, Utc};
use database::DbPool;
use database::models::{Comic as DatabaseComic, LogEntry};
use serde::Deserialize;
use shared::token_permissions;
use tracing::{Instrument, info_span};
use ts_rs::TS;

#[api_endpoint(method = "POST", path = "comicdata/advance")]
#[tracing::instrument(skip(pool, auth), fields(permissions = ?auth.authorities))]
pub async fn add_advance_comic(
    pool: web::Data<DbPool>,
    request: web::Json<AddAdvanceComicBody>,
    auth: AuthDetails,
) -> Result<Json<String>> {
    ensure_is_authorized(&auth, token_permissions::CAN_ADD_ADVANCE_COMIC)
        .map_err(error::ErrorForbidden)?;

    let AddAdvanceComicBody {
        token,
        comic_id,
        title,
        tagline,
        publish_date,
        is_accurate_publish_date,
        is_guest_comic,
        is_non_canon,
    } = request.into_inner();

    let mut transaction = pool
        .begin()
        .instrument(info_span!("Pool::begin"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let comic_id = comic_id.into_inner();
    if let Some(existing) = DatabaseComic::by_id(&mut *transaction, comic_id)
        .await
        .map_err(error::ErrorInternalServerError)?
    {
        if existing.hidden == 0 {
            return Err(error::ErrorConflict(format!(
                "Comic #{comic_id} is already published and cannot be used for an advance comic"
            )));
        }
    }

    DatabaseComic::insert_advance_comic(
        &mut *transaction,
        comic_id,
        &title,
        tagline.as_deref(),
        publish_date.map(|d| d.naive_utc()),
        is_accurate_publish_date.unwrap_or(false),
        is_guest_comic.unwrap_or(false),
        is_non_canon.unwrap_or(false),
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    LogEntry::log_action(
        &mut *transaction,
        token.to_string(),
        format!("Added advance comic #{comic_id} (\"{title}\")"),
        Some(comic_id),
        None,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    transaction
        .commit()
        .instrument(info_span!("Transaction::commit"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(Json(format!("Added advance comic #{comic_id}")))
}

#[api_endpoint(method = "GET", path = "comicdata/advance")]
#[tracing::instrument(skip(pool, auth), fields(permissions = ?auth.authorities))]
pub async fn list_advance_comics(
    pool: web::Data<DbPool>,
    query: web::Query<ListAdvanceComicsQuery>,
    auth: AuthDetails,
) -> Result<Json<Vec<AdvanceComicListItem>>> {
    ensure_is_authorized(&auth, token_permissions::HAS_VALID_TOKEN)
        .map_err(error::ErrorForbidden)?;

    let comics: Vec<AdvanceComicListItem> =
        DatabaseComic::all_hidden_with_mapping(&***pool, From::from)
            .await
            .map_err(error::ErrorInternalServerError)?;

    Ok(Json(comics))
}

#[api_endpoint(method = "POST", path = "comicdata/advance/run-updater")]
#[tracing::instrument(skip(trigger, auth), fields(permissions = ?auth.authorities))]
pub async fn run_comic_updater(
    trigger: web::Data<ComicUpdaterTrigger>,
    request: web::Json<RunComicUpdaterBody>,
    auth: AuthDetails,
) -> Result<Json<String>> {
    ensure_is_authorized(&auth, token_permissions::CAN_ADD_ADVANCE_COMIC)
        .map_err(error::ErrorForbidden)?;

    match trigger.request_run() {
        Ok(()) => Ok(Json(String::from(
            "Requested an immediate comic updater run",
        ))),
        Err(remaining) => Err(error::ErrorTooManyRequests(format!(
            "The comic updater last ran less than 5 minutes ago; try again in {} seconds",
            remaining.as_secs()
        ))),
    }
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AddAdvanceComicBody {
    pub token: Token,
    pub comic_id: ComicId,
    pub title: String,

    #[ts(optional)]
    pub tagline: Option<String>,
    #[ts(optional, type = "string")]
    pub publish_date: Option<DateTime<Utc>>,
    #[ts(optional)]
    pub is_accurate_publish_date: Option<bool>,
    #[ts(optional)]
    pub is_guest_comic: Option<bool>,
    #[ts(optional)]
    pub is_non_canon: Option<bool>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct ListAdvanceComicsQuery {
    // This is never read directly because it's used by the auth middleware only.
    // We still include it here so it ends up in the TS binding.
    #[expect(dead_code)]
    #[ts(optional)]
    pub token: Option<Token>,
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RunComicUpdaterBody {
    // This is never read directly because it's used by the auth middleware only.
    // We still include it here so it ends up in the TS binding.
    #[expect(dead_code)]
    pub token: Token,
}
