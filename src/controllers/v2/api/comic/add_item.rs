use crate::util::{andify_comma_string, ensure_is_authorized, ensure_is_valid};
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::AuthDetails;
use anyhow::anyhow;
use database::models::{Comic as DatabaseComic, Item as DatabaseItem, LogEntry, Occurrence};
use database::DbPool;
use parse_display::Display;
use semval::context::Context as ValidationContext;
use semval::Validate;
use serde::Deserialize;
use shared::token_permissions;
use std::fmt::Write;
use ts_rs::TS;

use crate::models::v2::{ComicId, ComicIdInvalidity, False, ItemType, Token, True};

#[allow(clippy::too_many_lines)]
pub(crate) async fn add_item(
    pool: web::Data<DbPool>,
    request: web::Json<AddItemToComicBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    ensure_is_authorized(&auth, token_permissions::CAN_ADD_ITEM_TO_COMIC)
        .map_err(error::ErrorForbidden)?;

    let request = request.into_inner();
    ensure_is_valid(&request).map_err(error::ErrorBadRequest)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    DatabaseComic::ensure_exists_by_id(&mut *transaction, request.comic_id.into_inner())
        .await
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

            let new_item_id = result.last_insert_id() as u16;

            LogEntry::log_action(
                &mut *transaction,
                request.token.to_string(),
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
                request.comic_id.into_inner(),
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

    // TODO: Turn off hasNoCast / hasNoLocation / hasNoStoryline flag if enabled

    Occurrence::create(&mut *transaction, id, request.comic_id.into_inner())
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
        request.token.to_string(),
        &action,
        Some(request.comic_id.into_inner()),
        Some(id),
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    transaction
        .commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(action))
}

pub(crate) async fn add_items(
    pool: web::Data<DbPool>,
    request: web::Json<AddItemsToComicBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    ensure_is_authorized(&auth, token_permissions::CAN_ADD_ITEM_TO_COMIC)
        .map_err(error::ErrorForbidden)?;

    let request = request.into_inner();
    ensure_is_valid(&request).map_err(error::ErrorBadRequest)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    DatabaseComic::ensure_exists_by_id(&mut *transaction, request.comic_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?;

    let mut items_added = String::new();
    for item in request.items {
        let item = DatabaseItem::by_id(&mut *transaction, item.item_id)
            .await
            .map_err(error::ErrorInternalServerError)?
            .ok_or_else(|| {
                error::ErrorBadRequest(anyhow!("Item #{} does not exist", item.item_id))
            })?;

        let occurrence_exists = Occurrence::occurrence_by_item_id_and_comic_id(
            &mut *transaction,
            item.id,
            request.comic_id.into_inner(),
        )
        .await
        .map_err(error::ErrorInternalServerError)?;

        if occurrence_exists {
            // In this multi-item-add scenario, we'll ignore existing items
            continue;
        }

        // TODO: Turn off hasNoCast / hasNoLocation / hasNoStoryline flag if enabled

        Occurrence::create(&mut *transaction, item.id, request.comic_id.into_inner())
            .await
            .map_err(error::ErrorInternalServerError)?;

        LogEntry::log_action(
            &mut *transaction,
            request.token.to_string(),
            format!(
                "Added {} #{} ({}) to comic #{}",
                item.r#type.as_str(),
                item.id,
                item.name,
                request.comic_id
            ),
            Some(request.comic_id.into_inner()),
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

    Ok(HttpResponse::Ok().body(if items_added.is_empty() {
        format!(
            "No new items added to comic {}; they were all already added",
            request.comic_id
        )
    } else {
        format!("Items {} added to comic {}", items_added, request.comic_id)
    }))
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AddItemToComicBody {
    pub token: Token,
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
                        _ => false,
                    }
                },
                AddItemToComicBodyInvalidity::ItemIdInvalid,
            )
            .invalidate_if(
                {
                    match &self.item {
                        ItemBody::New(new) => new.new_item_name.is_empty(),
                        _ => false,
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
    pub token: Token,
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
#[allow(variant_size_differences)]
pub enum ItemBody {
    New(NewItem),
    Existing(ExistingItem),
}

#[derive(Debug, Default, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExistingItem {
    pub new: False,
    pub item_id: u16,
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NewItem {
    pub new: True,
    pub new_item_name: String,
    pub new_item_type: ItemType,
}
