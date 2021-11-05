use crate::models::{ItemColor, Token};
use crate::util::ensure_is_authorized;
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::AuthDetails;
use anyhow::anyhow;
use database::models::{Item as DatabaseItem, LogEntry};
use database::DbPool;
use serde::Deserialize;
use shared::token_permissions;

#[allow(clippy::too_many_lines)]
pub(crate) async fn set_property(
    pool: web::Data<DbPool>,
    request: web::Json<SetItemPropertyBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    ensure_is_authorized(&auth, token_permissions::CAN_CHANGE_ITEM_DATA)
        .map_err(error::ErrorForbidden)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let old_item = DatabaseItem::by_id(&mut *transaction, request.item_id)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| {
            error::ErrorNotFound(anyhow!("No item with id {} exists", request.item_id))
        })?;

    match &*request.property {
        "name" => {
            DatabaseItem::update_name_by_id(&mut *transaction, request.item_id, &request.value)
                .await
                .map_err(error::ErrorInternalServerError)?;

            if old_item.name.is_empty() {
                LogEntry::log_action(
                    &mut *transaction,
                    request.token.to_string(),
                    format!(
                        "Set name of {} #{} to \"{}\"",
                        old_item.r#type, request.item_id, request.value
                    ),
                )
                .await
                .map_err(error::ErrorInternalServerError)?;
            } else {
                LogEntry::log_action(
                    &mut *transaction,
                    request.token.to_string(),
                    format!(
                        "Changed name of {} #{} from \"{}\" to \"{}\"",
                        old_item.r#type, request.item_id, old_item.name, request.value
                    ),
                )
                .await
                .map_err(error::ErrorInternalServerError)?;
            }
        }
        "shortName" => {
            DatabaseItem::update_short_name_by_id(
                &mut *transaction,
                request.item_id,
                &request.value,
            )
            .await
            .map_err(error::ErrorInternalServerError)?;

            if old_item.shortName.is_empty() {
                LogEntry::log_action(
                    &mut *transaction,
                    request.token.to_string(),
                    format!(
                        "Set shortName of {} #{} to \"{}\"",
                        old_item.r#type, request.item_id, request.value
                    ),
                )
                .await
                .map_err(error::ErrorInternalServerError)?;
            } else {
                LogEntry::log_action(
                    &mut *transaction,
                    request.token.to_string(),
                    format!(
                        "Changed shortName of {} #{} from \"{}\" to \"{}\"",
                        old_item.r#type, request.item_id, old_item.shortName, request.value
                    ),
                )
                .await
                .map_err(error::ErrorInternalServerError)?;
            }
        }
        "color" => {
            let old_color = ItemColor(
                old_item.Color_Red,
                old_item.Color_Green,
                old_item.Color_Blue,
            );
            let new_color: ItemColor = request.value.parse().map_err(error::ErrorBadRequest)?;

            DatabaseItem::update_color_by_id(
                &mut *transaction,
                request.item_id,
                new_color.0,
                new_color.1,
                new_color.2,
            )
            .await
            .map_err(error::ErrorInternalServerError)?;

            LogEntry::log_action(
                &mut *transaction,
                request.token.to_string(),
                format!(
                    "Changed color of {} #{} from \"{}\" to \"{}\"",
                    old_item.r#type, request.item_id, old_color, new_color
                ),
            )
            .await
            .map_err(error::ErrorInternalServerError)?;
        }
        property => {
            return Err(error::ErrorBadRequest(anyhow!(
                "No property named \"{}\"",
                property
            )))
        }
    }

    transaction
        .commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(format!(
        "Item property {} has been updated on item #{}",
        request.property, request.item_id
    )))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetItemPropertyBody {
    token: Token,
    #[serde(rename = "item")]
    item_id: u16,
    property: String,
    value: String,
}
