use crate::controllers::api::comic::ensure_comic_exists;
use crate::database::DbPool;
use crate::models::{token_permissions, ComicId, ComicIdInvalidity, Token};
use crate::util::{ensure_is_authorized, ensure_is_valid, log_action};
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::AuthDetails;
use parse_display::Display;
use semval::{context::Context as ValidationContext, Validate};
use serde::Deserialize;

pub(crate) async fn set_tagline(
    pool: web::Data<DbPool>,
    request: web::Json<SetTaglineBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    ensure_is_authorized(&auth, token_permissions::CAN_CHANGE_COMIC_DATA)
        .map_err(error::ErrorForbidden)?;

    ensure_is_valid(&*request).map_err(error::ErrorBadRequest)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    ensure_comic_exists(&mut *transaction, request.comic_id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let old_tagline = sqlx::query_scalar!(
        r#"
            SELECT tagline FROM `comic` WHERE id = ?
        "#,
        request.comic_id.into_inner()
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?;

    sqlx::query!(
        r#"
            UPDATE `comic`
            SET tagline = ?
            WHERE
                id = ?
        "#,
        request.tagline,
        request.comic_id.into_inner(),
    )
    .execute(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?;

    match old_tagline {
        Some(old_tagline) if !old_tagline.is_empty() => {
            log_action(
                &mut *transaction,
                request.token,
                format!(
                    "Changed tagline on comic #{} from \"{}\" to \"{}\"",
                    request.comic_id, old_tagline, request.tagline
                ),
            )
            .await
            .map_err(error::ErrorInternalServerError)?;
        }
        _ => {
            log_action(
                &mut *transaction,
                request.token,
                format!(
                    "Set tagline on comic #{} to \"{}\"",
                    request.comic_id, request.tagline
                ),
            )
            .await
            .map_err(error::ErrorInternalServerError)?;
        }
    }

    transaction
        .commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body("Tagline set or updated for comic"))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetTaglineBody {
    token: Token,
    comic_id: ComicId,
    tagline: String,
}

impl Validate for SetTaglineBody {
    type Invalidity = SetTaglineBodyInvalidity;

    fn validate(&self) -> semval::Result<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.comic_id, SetTaglineBodyInvalidity::ComicId)
            .into()
    }
}

#[derive(Copy, Clone, Debug, Display, Eq, PartialEq)]
pub(crate) enum SetTaglineBodyInvalidity {
    #[display("{0}")]
    ComicId(ComicIdInvalidity),
}
