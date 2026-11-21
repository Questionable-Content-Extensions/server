use crate::api::v3::models::AdvanceComicListItem;
use crate::models::{ComicId, Token};
use crate::util::{ComicUpdaterTrigger, TextTooLong, ensure_is_authorized, ensure_is_valid};
use actix_web::web::Json;
use actix_web::{Result, error, web};
use actix_web_grants::authorities::AuthDetails;
use api_macros::api_endpoint;
use chrono::{DateTime, Utc};
use database::DbPool;
use database::models::{
    COMIC_TAGLINE_MAX_CHARS, COMIC_TITLE_MAX_CHARS, Comic as DatabaseComic, LogEntry,
};
use semval::{Validate, ValidationResult};
use serde::Deserialize;
use shared::token_permissions;
use tracing::{Instrument, info_span};
use ts_rs::TS;

#[api_endpoint(method = "POST", path = "comicdata/advance")]
#[tracing::instrument(skip(pool, auth), fields(permissions = ?auth.authorities))]
pub async fn add_advance_comic(
    pool: web::Data<DbPool>,
    request: web::Json<AddAdvanceComicBody>,
    token: web::ReqData<Token>,
    auth: AuthDetails,
) -> Result<Json<String>> {
    ensure_is_authorized(&auth, token_permissions::CAN_ADD_ADVANCE_COMIC)
        .map_err(error::ErrorForbidden)?;
    ensure_is_valid(&*request).map_err(error::ErrorBadRequest)?;

    let token = *token;
    let AddAdvanceComicBody {
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
        && existing.hidden == 0
    {
        return Err(error::ErrorConflict(format!(
            "Comic #{comic_id} is already published and cannot be used for an advance comic"
        )));
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

impl Validate for AddAdvanceComicBody {
    type Invalidity = TextTooLong;
    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        TextTooLong::collect([
            TextTooLong::check("title", Some(&self.title), COMIC_TITLE_MAX_CHARS),
            TextTooLong::check("tagline", self.tagline.as_deref(), COMIC_TAGLINE_MAX_CHARS),
        ])
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn body(title: &str, tagline: &str) -> AddAdvanceComicBody {
        serde_json::from_value(json!({ "comicId": 5000, "title": title, "tagline": tagline }))
            .unwrap()
    }

    #[test]
    fn title_and_tagline_at_their_column_limits_are_valid() {
        let body = body(
            &"a".repeat(COMIC_TITLE_MAX_CHARS),
            &"a".repeat(COMIC_TAGLINE_MAX_CHARS),
        );

        assert!(ensure_is_valid(&body).is_ok());
    }

    #[test]
    fn an_overlong_title_and_tagline_are_rejected_naming_the_limits() {
        let body = body(
            &"a".repeat(COMIC_TITLE_MAX_CHARS + 1),
            &"a".repeat(COMIC_TAGLINE_MAX_CHARS + 1),
        );

        let error = ensure_is_valid(&body).unwrap_err().to_string();

        assert!(
            error.contains("title must be at most 255 characters"),
            "{error}"
        );
        assert!(
            error.contains("tagline must be at most 255 characters"),
            "{error}"
        );
    }
}
