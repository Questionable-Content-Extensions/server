use crate::models::{ComicId, ComicIdInvalidity, ItemId, ItemIdInvalidity, Token};
use crate::util::{ensure_is_authorized, ensure_is_valid};
use actix_web::web::Json;
use actix_web::{Result, error, web};
use actix_web_grants::authorities::AuthDetails;
use anyhow::anyhow;
use api_macros::api_endpoint;
use database::DbPool;
use database::models::{
    Comic as DatabaseComic, Item as DatabaseItem, LogEntry, Occurrence as DatabaseOccurrence,
};
use parse_display::Display;
use semval::{Validate, context::Context as ValidationContext};
use serde::Deserialize;
use shared::token_permissions;
use tracing::{Instrument, info_span};
use ts_rs::TS;

#[api_endpoint(method = "POST", path = "comicdata/removeitem")]
#[tracing::instrument(skip(pool, auth), fields(permissions = ?auth.authorities))]
pub async fn remove_item(
    pool: web::Data<DbPool>,
    request: web::Json<RemoveItemFromComicBody>,
    auth: AuthDetails,
) -> Result<Json<String>> {
    ensure_is_authorized(&auth, token_permissions::CAN_REMOVE_ITEM_FROM_COMIC)
        .map_err(error::ErrorForbidden)?;

    ensure_is_valid(&*request).map_err(error::ErrorBadRequest)?;

    let mut transaction = pool
        .begin()
        .instrument(info_span!("Pool::begin"))
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

    let item_id = request.item_id.into_inner();
    let comic_id = request.comic_id.into_inner();
    let result = DatabaseOccurrence::delete(&mut *transaction, item_id, comic_id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    if result.rows_affected() != 1 {
        return Err(error::ErrorNotFound(format!(
            "Could not delete item {item_id} from comic {comic_id}; the item is not in the comic"
        )));
    }

    let action = format!(
        "Removed {} #{} ({}) from comic #{}",
        item.r#type, item.id, item.name, request.comic_id
    );
    LogEntry::log_action(
        &mut *transaction,
        request.token.to_string(),
        &action,
        Some(comic_id),
        Some(item_id),
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    transaction
        .commit()
        .instrument(info_span!("Transaction::commit"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(Json(action))
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RemoveItemFromComicBody {
    token: Token,
    comic_id: ComicId,
    item_id: ItemId,
}

impl Validate for RemoveItemFromComicBody {
    type Invalidity = RemoveItemFromComicBodyInvalidity;

    fn validate(&self) -> semval::ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.comic_id, RemoveItemFromComicBodyInvalidity::ComicId)
            .validate_with(&self.item_id, RemoveItemFromComicBodyInvalidity::ItemId)
            .into()
    }
}

#[derive(Copy, Clone, Debug, Display, Eq, PartialEq)]
pub enum RemoveItemFromComicBodyInvalidity {
    #[display("{0}")]
    ComicId(ComicIdInvalidity),
    #[display("{0}")]
    ItemId(ItemIdInvalidity),
}
