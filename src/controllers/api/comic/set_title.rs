use crate::controllers::api::comic::ensure_comic_exists;
use crate::database::DbPool;
use crate::models::{token_permissions, ComicId, ComicIdInvalidity, Token};
use crate::util::{ensure_is_authorized, ensure_is_valid, log_action};
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::AuthDetails;
use parse_display::Display;
use semval::context::Context as ValidationContext;
use semval::Validate;
use serde::Deserialize;

pub(crate) async fn set_title(
    pool: web::Data<DbPool>,
    request: web::Json<SetTitleBody>,
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

    let old_title = sqlx::query_scalar!(
        r#"
            SELECT title FROM `comic` WHERE id = ?
        "#,
        request.comic_id.into_inner()
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?;

    sqlx::query!(
        r#"
            UPDATE `comic`
            SET title = ?
            WHERE
                id = ?
        "#,
        request.title,
        request.comic_id.into_inner(),
    )
    .execute(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?;

    if old_title.is_empty() {
        log_action(
            &mut *transaction,
            request.token,
            format!(
                "Set title on comic #{} to \"{}\"",
                request.comic_id, request.title
            ),
        )
        .await
        .map_err(error::ErrorInternalServerError)?;
    } else {
        log_action(
            &mut *transaction,
            request.token,
            format!(
                "Changed title on comic #{} from \"{}\" to \"{}\"",
                request.comic_id, old_title, request.title
            ),
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
    token: Token,
    comic_id: ComicId,
    title: String,
}

impl Validate for SetTitleBody {
    type Invalidity = SetTitleBodyInvalidity;

    fn validate(&self) -> semval::Result<Self::Invalidity> {
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
