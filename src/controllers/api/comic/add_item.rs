use crate::controllers::api::comic::ensure_comic_exists;
use crate::database::models::Item as DatabaseItem;
use crate::database::DbPool;
use crate::models::{token_permissions, ComicId, ComicIdInvalidity, Token};
use crate::util::{ensure_is_authorized, ensure_is_valid, log_action};
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::AuthDetails;
use anyhow::anyhow;
use parse_display::Display;
use semval::{context::Context as ValidationContext, Result as ValidationResult, Validate};
use serde::Deserialize;

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

    ensure_comic_exists(&mut *transaction, request.comic_id)
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

        let result = sqlx::query!(
            r#"
                INSERT INTO `items`
                    (name, shortName, type)
                VALUES
                    (?, ?, ?)
            "#,
            new_item_name,
            new_item_name,
            new_item_type,
        )
        .execute(&mut *transaction)
        .await
        .map_err(error::ErrorInternalServerError)?;

        let new_item_id = result.last_insert_id() as i16;

        log_action(
            &mut *transaction,
            request.token,
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
            shortName: new_item_name.clone(),
            r#type: new_item_type.clone(),
            Color_Blue: 127,
            Color_Green: 127,
            Color_Red: 127,
        }
    } else {
        let item = sqlx::query_as!(
            DatabaseItem,
            r#"
                SELECT * FROM `items` WHERE id = ?
            "#,
            request.item_id
        )
        .fetch_optional(&mut *transaction)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorBadRequest(anyhow!("Item does not exist")))?;

        let occurrence_exists = sqlx::query_scalar!(
            r#"
                SELECT COUNT(1) FROM `occurences`
                WHERE
                    items_id = ?
                AND
                    comic_id = ?
            "#,
            request.item_id,
            request.comic_id.into_inner(),
        )
        .fetch_one(&mut *transaction)
        .await
        .map_err(error::ErrorInternalServerError)?
            == 1;

        if occurrence_exists {
            return Err(error::ErrorBadRequest(anyhow!(
                "Item is already added to comic"
            )));
        }

        item
    };

    sqlx::query!(
        r#"
            INSERT INTO `occurences`
                (comic_id, items_id)
            VALUES
                (?, ?)
        "#,
        request.comic_id.into_inner(),
        request.item_id
    )
    .execute(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?;

    log_action(
        &mut *transaction,
        request.token,
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
    fn validate(&self) -> ValidationResult<Self::Invalidity> {
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
