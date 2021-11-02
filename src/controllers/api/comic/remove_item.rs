use crate::models::{ComicId, ComicIdInvalidity, ItemId, ItemIdInvalidity, Token};
use crate::util::{ensure_is_authorized, ensure_is_valid};
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::AuthDetails;
use anyhow::anyhow;
use database::models::{
    Comic as DatabaseComic, Item as DatabaseItem, LogEntry, Occurrence as DatabaseOccurrence,
};
use database::DbPool;
use parse_display::Display;
use semval::{context::Context as ValidationContext, Result as ValidationResult, Validate};
use serde::Deserialize;
use shared::token_permissions;

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

    let comic_exists =
        DatabaseComic::exists_by_id(&mut *transaction, request.comic_id.into_inner())
            .await
            .map_err(error::ErrorInternalServerError)?;

    if !comic_exists {
        return Err(error::ErrorBadRequest(anyhow!("Comic does not exist")));
    }

    let item = DatabaseItem::by_id(&mut *transaction, request.item_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorBadRequest(anyhow!("Item does not exist")))?;

    DatabaseOccurrence::delete(
        &mut *transaction,
        request.item_id.into_inner(),
        request.comic_id.into_inner(),
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    LogEntry::log_action(
        &mut *transaction,
        request.token.to_string(),
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
