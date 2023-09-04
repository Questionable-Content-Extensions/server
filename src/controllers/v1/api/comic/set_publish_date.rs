use crate::models::v1::{ComicId, ComicIdInvalidity, Token};
use crate::util::{ensure_is_authorized, ensure_is_valid, AddMonths};
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::AuthDetails;
use chrono::{DateTime, TimeZone, Utc};
use database::models::{Comic as DatabaseComic, LogEntry};
use database::DbPool;
use parse_display::Display;
use semval::{context::Context as ValidationContext, Validate};
use serde::Deserialize;
use shared::token_permissions;

pub(crate) async fn set_publish_date(
    pool: web::Data<DbPool>,
    request: web::Json<SetPublishDateBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    ensure_is_authorized(&auth, token_permissions::CAN_CHANGE_COMIC_DATA)
        .map_err(error::ErrorForbidden)?;

    ensure_is_valid(&*request).map_err(error::ErrorBadRequest)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    DatabaseComic::ensure_exists_by_id(&mut *transaction, request.comic_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?;

    let old_publish_date =
        DatabaseComic::publish_date_by_id(&mut *transaction, request.comic_id.into_inner())
            .await
            .map_err(error::ErrorInternalServerError)?;

    DatabaseComic::update_publish_date_by_id(
        &mut *transaction,
        request.comic_id.into_inner(),
        request.publish_date,
        request.is_accurate_publish_date,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    if let Some(old_publish_date) = old_publish_date {
        LogEntry::log_action(
            &mut *transaction,
            request.token.to_string(),
            format!(
                "Changed publish date on comic #{} from \"{}\" to \"{}\"",
                request.comic_id,
                Utc.from_utc_datetime(&old_publish_date)
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                request
                    .publish_date
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            ),
            Some(request.comic_id.into_inner()),
            None,
        )
        .await
        .map_err(error::ErrorInternalServerError)?;
    } else {
        LogEntry::log_action(
            &mut *transaction,
            request.token.to_string(),
            format!(
                "Set publish date on comic #{} to \"{}\"",
                request.comic_id,
                request
                    .publish_date
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            ),
            Some(request.comic_id.into_inner()),
            None,
        )
        .await
        .map_err(error::ErrorInternalServerError)?;
    }

    transaction
        .commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetPublishDateBody {
    token: Token,
    comic_id: ComicId,
    publish_date: DateTime<Utc>,
    is_accurate_publish_date: bool,
}

impl Validate for SetPublishDateBody {
    type Invalidity = SetPublishDateBodyInvalidity;

    fn validate(&self) -> semval::ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.comic_id, SetPublishDateBodyInvalidity::ComicId)
            .invalidate_if(
                self.publish_date < Utc.ymd(2003, 8, 1).and_hms(0, 0, 0)
                    || self.publish_date > Utc::now().add_months(1),
                SetPublishDateBodyInvalidity::PublishDate,
            )
            .into()
    }
}

#[derive(Copy, Clone, Debug, Display, Eq, PartialEq)]
pub(crate) enum SetPublishDateBodyInvalidity {
    #[display("{0}")]
    ComicId(ComicIdInvalidity),
    #[display("Provided publish date must be after the comic was started and no later than one month from today's date")]
    PublishDate,
}
