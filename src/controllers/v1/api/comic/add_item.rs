use crate::models::v1::{ComicId, ComicIdInvalidity, Token};
use crate::util::{ensure_is_authorized, ensure_is_valid};
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::AuthDetails;
use anyhow::anyhow;
use database::models::{Comic as DatabaseComic, Item as DatabaseItem, LogEntry, Occurrence};
use database::DbPool;
use parse_display::Display;
use semval::{context::Context as ValidationContext, Validate};
use serde::Deserialize;
use shared::token_permissions;

#[allow(clippy::too_many_lines)]
pub(crate) async fn add_item(
    pool: web::Data<DbPool>,
    request: web::Json<AddItemToComicBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    const CREATE_NEW_ITEM_ID: i16 = -1;

    ensure_is_authorized(&auth, token_permissions::CAN_ADD_ITEM_TO_COMIC)
        .map_err(error::ErrorForbidden)?;

    ensure_is_valid(&*request).map_err(error::ErrorBadRequest)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    DatabaseComic::ensure_exists_by_id(&mut *transaction, request.comic_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?;

    let item = if request.item_id == CREATE_NEW_ITEM_ID {
        let new_item_name = request.new_item_name.as_ref().ok_or_else(|| {
            error::ErrorBadRequest(anyhow!(
                "New Item request without providing newItemName value"
            ))
        })?;
        let new_item_type = request.new_item_type.as_ref().ok_or_else(|| {
            error::ErrorBadRequest(anyhow!(
                "New Item request without providing newItemType value"
            ))
        })?;

        let result = DatabaseItem::create(
            &mut *transaction,
            new_item_name,
            new_item_name,
            AsRef::<str>::as_ref(new_item_type)
                .try_into()
                .map_err(|e| error::ErrorBadRequest(anyhow!("Invalid item type: {}", e)))?,
        )
        .await
        .map_err(error::ErrorInternalServerError)?;

        let new_item_id = result.last_insert_id() as u16;

        LogEntry::log_action(
            &mut *transaction,
            request.token.to_string(),
            format!(
                "Created {} #{} ({})",
                new_item_type, new_item_id, new_item_name
            ),
        )
        .await
        .map_err(error::ErrorInternalServerError)?;

        DatabaseItem {
            id: new_item_id,
            name: new_item_name.clone(),
            short_name: new_item_name.clone(),
            r#type: new_item_type.clone(),
            color_blue: 127,
            color_green: 127,
            color_red: 127,
        }
    } else {
        let item_id = request.item_id as u16;
        let item = DatabaseItem::by_id(&mut *transaction, item_id)
            .await
            .map_err(error::ErrorInternalServerError)?
            .ok_or_else(|| error::ErrorBadRequest(anyhow!("Item does not exist")))?;

        let occurrence_exists = Occurrence::occurrence_by_item_id_and_comic_id(
            &mut *transaction,
            item_id,
            request.comic_id.into_inner(),
        )
        .await
        .map_err(error::ErrorInternalServerError)?;

        if occurrence_exists {
            return Err(error::ErrorBadRequest(anyhow!(
                "Item is already added to comic"
            )));
        }

        item
    };

    // TODO: Turn off hasNoCast / hasNoLocation / hasNoStoryline flag if enabled

    Occurrence::create(&mut *transaction, item.id, request.comic_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?;

    LogEntry::log_action(
        &mut *transaction,
        request.token.to_string(),
        format!(
            "Added {} #{} ({}) to comic #{}",
            item.r#type, item.id, item.name, request.comic_id
        ),
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    transaction
        .commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body("Item added to comic"))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AddItemToComicBody {
    token: Token,
    comic_id: ComicId,
    item_id: i16,
    #[serde(default)]
    new_item_name: Option<String>,
    #[serde(default)]
    new_item_type: Option<String>,
}

impl Validate for AddItemToComicBody {
    type Invalidity = AddItemToComicBodyInvalidity;
    fn validate(&self) -> semval::ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.comic_id, AddItemToComicBodyInvalidity::ComicId)
            .invalidate_if(
                self.item_id < 1 && self.item_id != -1,
                AddItemToComicBodyInvalidity::ItemIdInvalid,
            )
            .invalidate_if(
                self.item_id >= 1 && self.new_item_name.is_some(),
                AddItemToComicBodyInvalidity::NewItemNameUsedWithoutCreateNewItemId,
            )
            .invalidate_if(
                self.item_id == -1 && self.new_item_name.is_none(),
                AddItemToComicBodyInvalidity::NewItemNameMissingWithCreateNewItemId,
            )
            .invalidate_if(
                self.item_id >= 1 && self.new_item_type.is_some(),
                AddItemToComicBodyInvalidity::NewItemTypeUsedWithoutCreateNewItemId,
            )
            .invalidate_if(
                self.item_id == -1 && self.new_item_type.is_none(),
                AddItemToComicBodyInvalidity::NewItemTypeMissingWithCreateNewItemId,
            )
            .into()
    }
}

#[derive(Copy, Clone, Debug, Display, Eq, PartialEq)]
pub(crate) enum AddItemToComicBodyInvalidity {
    #[display("{0}")]
    ComicId(ComicIdInvalidity),
    #[display(
        "itemId must be either -1 (for a new item) or a value larger than 0 (for an existing item)"
    )]
    ItemIdInvalid,
    #[display("newItemName value given when itemId was not -1")]
    NewItemNameUsedWithoutCreateNewItemId,
    #[display("newItemName value not given when itemId was -1")]
    NewItemNameMissingWithCreateNewItemId,
    #[display("newItemType value given when itemId was not -1")]
    NewItemTypeUsedWithoutCreateNewItemId,
    #[display("newItemType value not given when itemId was -1")]
    NewItemTypeMissingWithCreateNewItemId,
}
