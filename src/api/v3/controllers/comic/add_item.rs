use crate::api::v3::controllers::comic::patch_comic::{FlagType, update_flag};
use crate::api::v3::models::ItemType;
use crate::models::{ComicId, ComicIdInvalidity, False, Token, True};
use crate::util::{andify_comma_string, ensure_is_authorized, ensure_is_valid};
use actix_web::web::Json;
use actix_web::{Result, error, web};
use actix_web_grants::authorities::AuthDetails;
use anyhow::anyhow;
use api_macros::api_endpoint;
use database::DbPool;
use database::models::{Comic as DatabaseComic, Item as DatabaseItem, LogEntry, Occurrence};
use parse_display::Display;
use semval::Validate;
use semval::context::Context as ValidationContext;
use serde::Deserialize;
use shared::token_permissions;
use std::fmt::Write;
use tracing::{Instrument, info_span};
use ts_rs::TS;

#[api_endpoint(method = "POST", path = "comicdata/additem")]
#[tracing::instrument(skip(pool,  auth), fields(permissions = ?auth.authorities))]
#[expect(clippy::too_many_lines)]
pub async fn add_item(
    pool: web::Data<DbPool>,
    request: web::Json<AddItemToComicBody>,
    token: web::ReqData<Token>,
    auth: AuthDetails,
) -> Result<Json<String>> {
    ensure_is_authorized(&auth, token_permissions::CAN_ADD_ITEM_TO_COMIC)
        .map_err(error::ErrorForbidden)?;

    let token = *token;
    let request = request.into_inner();
    ensure_is_valid(&request).map_err(error::ErrorBadRequest)?;

    let mut transaction = pool
        .begin()
        .instrument(info_span!("Pool::begin"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let comic_id = request.comic_id.into_inner();
    DatabaseComic::ensure_exists_by_id(&mut *transaction, comic_id)
        .await
        .map_err(error::ErrorInternalServerError)?;
    let comic = DatabaseComic::by_id(&mut *transaction, comic_id)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| anyhow!("Comic does not exist despite being ensured to exist!"))
        .map_err(error::ErrorInternalServerError)?;

    let (id, name, r#type) = match request.item {
        ItemBody::New(new) => {
            let result = DatabaseItem::create(
                &mut *transaction,
                &new.new_item_name,
                &new.new_item_name,
                (new.new_item_type).into(),
            )
            .await
            .map_err(error::ErrorInternalServerError)?;

            let new_item_id =
                u16::try_from(result.last_insert_id()).expect("new item ID fits in u16");

            LogEntry::log_action(
                &mut *transaction,
                token.to_string(),
                format!(
                    "Created {} #{} ({})",
                    new.new_item_type.as_str(),
                    new_item_id,
                    new.new_item_name
                ),
                None,
                Some(new_item_id),
            )
            .await
            .map_err(error::ErrorInternalServerError)?;

            if let Some(start_comic_id) =
                default_start_comic_id_for_new_item(new.new_item_type, comic_id)
            {
                DatabaseItem::update_start_comic_id_by_id(
                    &mut *transaction,
                    new_item_id,
                    start_comic_id,
                )
                .await
                .map_err(error::ErrorInternalServerError)?;

                LogEntry::log_action(
                    &mut *transaction,
                    token.to_string(),
                    format!("Changed startComicId of item #{new_item_id} to {start_comic_id}"),
                    None,
                    Some(new_item_id),
                )
                .await
                .map_err(error::ErrorInternalServerError)?;
            }

            (new_item_id, new.new_item_name, new.new_item_type)
        }
        ItemBody::Existing(existing) => {
            let item = DatabaseItem::by_id(&mut *transaction, existing.item_id)
                .await
                .map_err(error::ErrorInternalServerError)?
                .ok_or_else(|| error::ErrorBadRequest(anyhow!("Item does not exist")))?;

            let occurrence_exists = Occurrence::occurrence_by_item_id_and_comic_id(
                &mut *transaction,
                existing.item_id,
                comic_id,
            )
            .await
            .map_err(error::ErrorInternalServerError)?;

            if occurrence_exists {
                return Err(error::ErrorBadRequest(anyhow!(
                    "Item is already added to comic"
                )));
            }

            (
                item.id,
                item.name,
                (&*item.r#type)
                    .try_into()
                    .map_err(error::ErrorInternalServerError)?,
            )
        }
    };

    let (flagtype, flag_needs_update) = match r#type {
        ItemType::Cast => (FlagType::HasNoCast, comic.has_no_cast != 0),
        ItemType::Location => (FlagType::HasNoLocation, comic.has_no_location != 0),
        ItemType::Storyline => (FlagType::HasNoStoryline, comic.has_no_storyline != 0),
    };
    if flag_needs_update {
        update_flag(flagtype, false, request.comic_id, token, &mut transaction).await?;
    }

    Occurrence::create(&mut *transaction, id, comic_id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let action = format!(
        "Added {} #{} ({}) to comic #{}",
        r#type.as_str(),
        id,
        name,
        request.comic_id
    );
    LogEntry::log_action(
        &mut *transaction,
        token.to_string(),
        &action,
        Some(comic_id),
        Some(id),
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

#[api_endpoint(method = "POST", path = "comicdata/additems")]
pub async fn add_items(
    pool: web::Data<DbPool>,
    request: web::Json<AddItemsToComicBody>,
    token: web::ReqData<Token>,
    auth: AuthDetails,
) -> Result<Json<String>> {
    ensure_is_authorized(&auth, token_permissions::CAN_ADD_ITEM_TO_COMIC)
        .map_err(error::ErrorForbidden)?;

    let token = *token;
    let request = request.into_inner();
    ensure_is_valid(&request).map_err(error::ErrorBadRequest)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let comic_id = request.comic_id.into_inner();
    DatabaseComic::ensure_exists_by_id(&mut *transaction, comic_id)
        .await
        .map_err(error::ErrorInternalServerError)?;
    let mut comic = DatabaseComic::by_id(&mut *transaction, comic_id)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| anyhow!("Comic does not exist despite being ensured to exist!"))
        .map_err(error::ErrorInternalServerError)?;

    let mut items_added = String::new();
    for item in request.items {
        let item = DatabaseItem::by_id(&mut *transaction, item.item_id)
            .await
            .map_err(error::ErrorInternalServerError)?
            .ok_or_else(|| {
                error::ErrorBadRequest(anyhow!("Item #{} does not exist", item.item_id))
            })?;

        let occurrence_exists =
            Occurrence::occurrence_by_item_id_and_comic_id(&mut *transaction, item.id, comic_id)
                .await
                .map_err(error::ErrorInternalServerError)?;

        if occurrence_exists {
            // In this multi-item-add scenario, we'll ignore existing items
            continue;
        }

        let (flagtype, flag_needs_update, flag) =
            match ItemType::try_from(&*item.r#type).map_err(error::ErrorInternalServerError)? {
                ItemType::Cast => (
                    FlagType::HasNoCast,
                    comic.has_no_cast != 0,
                    &mut comic.has_no_cast,
                ),
                ItemType::Location => (
                    FlagType::HasNoLocation,
                    comic.has_no_location != 0,
                    &mut comic.has_no_location,
                ),
                ItemType::Storyline => (
                    FlagType::HasNoStoryline,
                    comic.has_no_storyline != 0,
                    &mut comic.has_no_storyline,
                ),
            };
        if flag_needs_update {
            update_flag(flagtype, false, request.comic_id, token, &mut transaction).await?;
            *flag = 0;
        }

        Occurrence::create(&mut *transaction, item.id, comic_id)
            .await
            .map_err(error::ErrorInternalServerError)?;

        LogEntry::log_action(
            &mut *transaction,
            token.to_string(),
            format!(
                "Added {} #{} ({}) to comic #{}",
                item.r#type.as_str(),
                item.id,
                item.name,
                request.comic_id
            ),
            Some(comic_id),
            Some(item.id),
        )
        .await
        .map_err(error::ErrorInternalServerError)?;

        if !items_added.is_empty() {
            items_added.push_str(", ");
        }
        write!(&mut items_added, "{} {}", item.r#type, item.name).unwrap();
    }
    andify_comma_string(&mut items_added);

    transaction
        .commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(Json(if items_added.is_empty() {
        format!(
            "No new items added to comic {}; they were all already added",
            request.comic_id
        )
    } else {
        format!("Items {} added to comic {}", items_added, request.comic_id)
    }))
}

/// Storylines have no "attached = belongs" equivalence like cast/location do
/// (their membership is governed by `startComicId`/`endComicId`, not
/// attachment), so creating one here must also default its lifecycle start
/// to the comic it's being added from, or it would never show up as active
/// anywhere until a second editor trip through the item editor.
fn default_start_comic_id_for_new_item(item_type: ItemType, comic_id: u16) -> Option<u16> {
    (item_type == ItemType::Storyline).then_some(comic_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_storyline_defaults_start_comic_id_to_the_comic_it_was_added_from() {
        assert_eq!(
            default_start_comic_id_for_new_item(ItemType::Storyline, 42),
            Some(42)
        );
    }

    #[test]
    fn new_cast_has_no_default_start_comic_id() {
        assert_eq!(
            default_start_comic_id_for_new_item(ItemType::Cast, 42),
            None
        );
    }

    #[test]
    fn new_location_has_no_default_start_comic_id() {
        assert_eq!(
            default_start_comic_id_for_new_item(ItemType::Location, 42),
            None
        );
    }
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AddItemToComicBody {
    pub comic_id: ComicId,
    #[serde(flatten)]
    pub item: ItemBody,
}

impl Validate for AddItemToComicBody {
    type Invalidity = AddItemToComicBodyInvalidity;
    fn validate(&self) -> semval::ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.comic_id, AddItemToComicBodyInvalidity::ComicId)
            .invalidate_if(
                {
                    match &self.item {
                        ItemBody::Existing(existing) => existing.item_id < 1,
                        ItemBody::New(_) => false,
                    }
                },
                AddItemToComicBodyInvalidity::ItemIdInvalid,
            )
            .invalidate_if(
                {
                    match &self.item {
                        ItemBody::New(new) => new.new_item_name.is_empty(),
                        ItemBody::Existing(_) => false,
                    }
                },
                AddItemToComicBodyInvalidity::EmptyNewItemName,
            )
            .into()
    }
}

#[derive(Copy, Clone, Debug, Display, Eq, PartialEq)]
pub enum AddItemToComicBodyInvalidity {
    #[display("{0}")]
    ComicId(ComicIdInvalidity),
    #[display(
        "itemId must be either -1 (for a new item) or a value larger than 0 (for an existing item)"
    )]
    ItemIdInvalid,
    #[display("newItemName value is empty")]
    EmptyNewItemName,
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AddItemsToComicBody {
    pub comic_id: ComicId,
    pub items: Vec<ExistingItem>,
}

impl Validate for AddItemsToComicBody {
    type Invalidity = AddItemsToComicBodyInvalidity;
    fn validate(&self) -> semval::ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.comic_id, AddItemsToComicBodyInvalidity::ComicId)
            .invalidate_if(
                'invalid: {
                    for item in &self.items {
                        if item.item_id < 1 {
                            break 'invalid true;
                        }
                    }
                    false
                },
                AddItemsToComicBodyInvalidity::ItemIdInvalid,
            )
            .into()
    }
}

#[derive(Copy, Clone, Debug, Display, Eq, PartialEq)]
pub enum AddItemsToComicBodyInvalidity {
    #[display("{0}")]
    ComicId(ComicIdInvalidity),
    #[display("itemIds must all be a value larger than 0")]
    ItemIdInvalid,
}

#[derive(Debug, Deserialize, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum ItemBody {
    New(NewItem),
    Existing(ExistingItem),
}

#[derive(Debug, Default, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExistingItem {
    #[expect(
        dead_code,
        reason = "discriminator field for untagged enum deserialization"
    )]
    pub new: False,
    pub item_id: u16,
}

#[expect(
    clippy::struct_field_names,
    reason = "field names match the serialized API field names"
)]
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NewItem {
    #[expect(
        dead_code,
        reason = "discriminator field for untagged enum deserialization"
    )]
    pub new: True,
    pub new_item_name: String,
    pub new_item_type: ItemType,
}
