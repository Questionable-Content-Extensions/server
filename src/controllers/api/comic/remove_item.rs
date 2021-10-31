use crate::database::models::Item as DatabaseItem;
use crate::database::DbPool;
use crate::models::{
    token_permissions, ComicId, ComicIdInvalidity, ItemId, ItemIdInvalidity, Token,
};
use crate::util::{ensure_is_authorized, ensure_is_valid, log_action};
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::AuthDetails;
use anyhow::anyhow;
use parse_display::Display;
use semval::{context::Context as ValidationContext, Result as ValidationResult, Validate};
use serde::Deserialize;

pub(crate) async fn remove_item(
    pool: web::Data<DbPool>,
    request: web::Json<RemoveItemFromComicBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    ensure_is_authorized(&auth, token_permissions::CAN_REMOVE_ITEM_FROM_COMIC)
        .map_err(error::ErrorForbidden)?;

    ensure_is_valid(&*request).map_err(error::ErrorBadRequest)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let comic_exists = sqlx::query_scalar!(
        r#"
            SELECT COUNT(1) FROM `comic`
            WHERE
                id = ?
        "#,
        request.comic_id.into_inner(),
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?
        == 1;

    if !comic_exists {
        return Err(error::ErrorBadRequest(anyhow!("Comic does not exist")));
    }

    let item = sqlx::query_as!(
        DatabaseItem,
        r#"
            SELECT * FROM `items` WHERE id = ?
        "#,
        request.item_id.into_inner()
    )
    .fetch_optional(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?
    .ok_or_else(|| error::ErrorBadRequest(anyhow!("Item does not exist")))?;

    sqlx::query!(
        r#"
            DELETE FROM `occurences`
            WHERE
                items_id = ?
            AND
                comic_id = ?
        "#,
        request.item_id.into_inner(),
        request.comic_id.into_inner(),
    )
    .execute(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?;

    log_action(
        &mut *transaction,
        request.token,
        format!(
            "Removed {} #{} ({}) from comic #{}",
            item.r#type, item.id, item.name, request.comic_id
        ),
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    transaction
        .commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body("Item removed from comic"))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RemoveItemFromComicBody {
    token: Token,
    comic_id: ComicId,
    item_id: ItemId,
}

impl Validate for RemoveItemFromComicBody {
    type Invalidity = RemoveItemFromComicBodyInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.comic_id, RemoveItemFromComicBodyInvalidity::ComicId)
            .validate_with(&self.item_id, RemoveItemFromComicBodyInvalidity::ItemId)
            .into()
    }
}

#[derive(Copy, Clone, Debug, Display, Eq, PartialEq)]
pub(crate) enum RemoveItemFromComicBodyInvalidity {
    #[display("{0}")]
    ComicId(ComicIdInvalidity),
    #[display("{0}")]
    ItemId(ItemIdInvalidity),
}
