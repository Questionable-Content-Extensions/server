use crate::models::{ComicId, ComicIdInvalidity, Token};
use crate::util::{ensure_is_authorized, ensure_is_valid};
use actix_web::{HttpResponse, Result, error, web};
use actix_web_grants::authorities::AuthDetails;
use database::DbPool;
use database::models::{Comic as DatabaseComic, LogEntry};
use parse_display::Display;
use semval::{Validate, context::Context as ValidationContext};
use serde::Deserialize;
use shared::token_permissions;

pub(crate) async fn set_title(
    pool: web::Data<DbPool>,
    request: web::Json<SetTitleBody>,
    token: web::ReqData<Token>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    ensure_is_authorized(&auth, token_permissions::CAN_CHANGE_COMIC_DATA)
        .map_err(error::ErrorForbidden)?;

    let token = *token;

    ensure_is_valid(&*request).map_err(error::ErrorBadRequest)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    DatabaseComic::ensure_exists_by_id(&mut *transaction, request.comic_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?;

    let old_title = DatabaseComic::title_by_id(&mut *transaction, request.comic_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?
        .expect("Due to ensure_exists_by_id call, this should never be None");

    DatabaseComic::update_title_by_id(
        &mut *transaction,
        request.comic_id.into_inner(),
        &request.title,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    if old_title.is_empty() {
        LogEntry::log_action(
            &mut *transaction,
            token.to_string(),
            format!(
                "Set title on comic #{} to \"{}\"",
                request.comic_id, request.title
            ),
            Some(request.comic_id.into_inner()),
            None,
        )
        .await
        .map_err(error::ErrorInternalServerError)?;
    } else {
        LogEntry::log_action(
            &mut *transaction,
            token.to_string(),
            format!(
                "Changed title on comic #{} from \"{}\" to \"{}\"",
                request.comic_id, old_title, request.title
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

    Ok(HttpResponse::Ok().body("Title set or updated for comic"))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetTitleBody {
    #[expect(dead_code, reason = "required in the request body for legacy clients")]
    token: Token,
    comic_id: ComicId,
    title: String,
}

impl Validate for SetTitleBody {
    type Invalidity = SetTitleBodyInvalidity;

    fn validate(&self) -> semval::ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.comic_id, SetTitleBodyInvalidity::ComicId)
            .into()
    }
}

#[derive(Copy, Clone, Debug, Display, Eq, PartialEq)]
pub(crate) enum SetTitleBodyInvalidity {
    #[display("{0}")]
    ComicId(ComicIdInvalidity),
}
