use crate::models::v1::{ComicId, ComicIdInvalidity, Token};
use crate::util::{ensure_is_authorized, ensure_is_valid};
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::AuthDetails;
use database::models::{Comic as DatabaseComic, LogEntry};
use database::DbPool;
use parse_display::Display;
use semval::{context::Context as ValidationContext, Validate};
use serde::Deserialize;
use shared::token_permissions;

pub(crate) async fn set_guest(
    pool: web::Data<DbPool>,
    request: web::Json<SetFlagBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    set_flag(pool, request, auth, FlagType::IsGuestComic).await?;

    Ok(HttpResponse::Ok().body("Guest comic set or updated for comic"))
}

pub(crate) async fn set_non_canon(
    pool: web::Data<DbPool>,
    request: web::Json<SetFlagBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    set_flag(pool, request, auth, FlagType::IsNonCanon).await?;

    Ok(HttpResponse::Ok().body("Non-canon set or updated for comic"))
}

pub(crate) async fn set_no_cast(
    pool: web::Data<DbPool>,
    request: web::Json<SetFlagBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    set_flag(pool, request, auth, FlagType::HasNoCast).await?;

    Ok(HttpResponse::Ok().body("No cast set or updated for comic"))
}

pub(crate) async fn set_no_location(
    pool: web::Data<DbPool>,
    request: web::Json<SetFlagBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    set_flag(pool, request, auth, FlagType::HasNoLocation).await?;

    Ok(HttpResponse::Ok().body("No location set or updated for comic"))
}

pub(crate) async fn set_no_storyline(
    pool: web::Data<DbPool>,
    request: web::Json<SetFlagBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    set_flag(pool, request, auth, FlagType::HasNoStoryline).await?;

    Ok(HttpResponse::Ok().body("No storyline set or updated for comic"))
}

pub(crate) async fn set_no_title(
    pool: web::Data<DbPool>,
    request: web::Json<SetFlagBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    set_flag(pool, request, auth, FlagType::HasNoTitle).await?;

    Ok(HttpResponse::Ok().body("No title set or updated for comic"))
}

pub(crate) async fn set_no_tagline(
    pool: web::Data<DbPool>,
    request: web::Json<SetFlagBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    set_flag(pool, request, auth, FlagType::HasNoTagline).await?;

    Ok(HttpResponse::Ok().body("No tagline set or updated for comic"))
}

#[allow(clippy::too_many_lines)]
async fn set_flag(
    pool: web::Data<DbPool>,
    request: web::Json<SetFlagBody>,
    auth: AuthDetails,
    flag: FlagType,
) -> Result<()> {
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

    let (true_value_log_text, false_value_log_text, sql_result) = match flag {
        FlagType::IsGuestComic => (
            "to be a guest comic",
            "to be a Jeph comic",
            DatabaseComic::update_is_guest_comic_by_id(
                &mut *transaction,
                request.comic_id.into_inner(),
                request.flag_value,
            )
            .await,
        ),
        FlagType::IsNonCanon => (
            "to be non-canon",
            "to be canon",
            DatabaseComic::update_is_non_canon_by_id(
                &mut *transaction,
                request.comic_id.into_inner(),
                request.flag_value,
            )
            .await,
        ),
        FlagType::HasNoCast => (
            "to have no cast",
            "to have cast",
            DatabaseComic::update_has_no_cast_by_id(
                &mut *transaction,
                request.comic_id.into_inner(),
                request.flag_value,
            )
            .await,
        ),
        FlagType::HasNoLocation => (
            "to have no locations",
            "to have locations",
            DatabaseComic::update_has_no_location_by_id(
                &mut *transaction,
                request.comic_id.into_inner(),
                request.flag_value,
            )
            .await,
        ),
        FlagType::HasNoStoryline => (
            "to have no storylines",
            "to have storylines",
            DatabaseComic::update_has_no_storyline_by_id(
                &mut *transaction,
                request.comic_id.into_inner(),
                request.flag_value,
            )
            .await,
        ),
        FlagType::HasNoTitle => (
            "to have no title",
            "to have a title",
            DatabaseComic::update_has_no_title_by_id(
                &mut *transaction,
                request.comic_id.into_inner(),
                request.flag_value,
            )
            .await,
        ),
        FlagType::HasNoTagline => (
            "to have no tagline",
            "to have a tagline",
            DatabaseComic::update_has_no_tagline_by_id(
                &mut *transaction,
                request.comic_id.into_inner(),
                request.flag_value,
            )
            .await,
        ),
    };

    sql_result.map_err(error::ErrorInternalServerError)?;

    LogEntry::log_action(
        &mut *transaction,
        request.token.to_string(),
        format!(
            "Set comic #{} {}",
            request.comic_id,
            if request.flag_value {
                true_value_log_text
            } else {
                false_value_log_text
            }
        ),
        Some(request.comic_id.into_inner()),
        None,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    transaction
        .commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(())
}

pub enum FlagType {
    IsGuestComic,
    IsNonCanon,
    HasNoCast,
    HasNoLocation,
    HasNoStoryline,
    HasNoTitle,
    HasNoTagline,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetFlagBody {
    token: Token,
    comic_id: ComicId,
    flag_value: bool,
}

impl Validate for SetFlagBody {
    type Invalidity = SetFlagBodyInvalidity;

    fn validate(&self) -> semval::ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.comic_id, SetFlagBodyInvalidity::ComicId)
            .into()
    }
}

#[derive(Copy, Clone, Debug, Display, Eq, PartialEq)]
pub(crate) enum SetFlagBodyInvalidity {
    #[display("{0}")]
    ComicId(ComicIdInvalidity),
}
