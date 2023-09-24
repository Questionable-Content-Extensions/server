use crate::api::v3::models::{ItemColor, ItemType};
use crate::models::{ItemId, Token};
use crate::util::{andify_comma_string, ensure_is_authorized};
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::AuthDetails;
use anyhow::anyhow;
use database::models::{Item as DatabaseItem, LogEntry};
use database::DbPool;
use serde::Deserialize;
use shared::token_permissions;
use tracing::{info_span, Instrument};
use ts_rs::TS;

#[tracing::instrument(skip(pool, auth), fields(permissions = ?auth.permissions))]
#[allow(clippy::too_many_lines)]
pub async fn patch_item(
    pool: web::Data<DbPool>,
    request: web::Json<PatchItemBody>,
    item_id: web::Path<ItemId>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    ensure_is_authorized(&auth, token_permissions::CAN_CHANGE_ITEM_DATA)
        .map_err(error::ErrorForbidden)?;

    let item_id = item_id.into_inner().into_inner();

    let mut transaction = pool
        .begin()
        .instrument(info_span!("Pool::begin"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let old_item = DatabaseItem::by_id(&mut *transaction, item_id)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound(anyhow!("No item with id {} exists", item_id)))?;

    let PatchItemBody {
        token,
        name,
        short_name,
        color,
        r#type,
    } = request.into_inner();

    let mut updated = Vec::with_capacity(3);

    if let Some(name) = &name {
        DatabaseItem::update_name_by_id(&mut *transaction, item_id, name)
            .await
            .map_err(error::ErrorInternalServerError)?;

        if old_item.name.is_empty() {
            LogEntry::log_action(
                &mut *transaction,
                token.to_string(),
                format!(
                    "Set name of {} #{} to \"{}\"",
                    old_item.r#type, item_id, name
                ),
                None,
                Some(item_id),
            )
            .await
            .map_err(error::ErrorInternalServerError)?;
        } else {
            LogEntry::log_action(
                &mut *transaction,
                token.to_string(),
                format!(
                    "Changed name of {} #{} from \"{}\" to \"{}\"",
                    old_item.r#type, item_id, old_item.name, name
                ),
                None,
                Some(item_id),
            )
            .await
            .map_err(error::ErrorInternalServerError)?;
        }

        updated.push("name")
    }

    if let Some(short_name) = short_name {
        DatabaseItem::update_short_name_by_id(&mut *transaction, item_id, &short_name)
            .await
            .map_err(error::ErrorInternalServerError)?;

        if old_item.short_name.is_empty() {
            LogEntry::log_action(
                &mut *transaction,
                token.to_string(),
                format!(
                    "Set shortName of {} #{} to \"{}\"",
                    old_item.r#type, item_id, short_name
                ),
                None,
                Some(item_id),
            )
            .await
            .map_err(error::ErrorInternalServerError)?;
        } else {
            LogEntry::log_action(
                &mut *transaction,
                token.to_string(),
                format!(
                    "Changed shortName of {} #{} from \"{}\" to \"{}\"",
                    old_item.r#type, item_id, old_item.short_name, short_name
                ),
                None,
                Some(item_id),
            )
            .await
            .map_err(error::ErrorInternalServerError)?;
        }

        updated.push("short name")
    }

    if let Some(color) = color {
        let old_color = ItemColor(
            old_item.color_red,
            old_item.color_green,
            old_item.color_blue,
        );
        let new_color: ItemColor = color.parse().map_err(error::ErrorBadRequest)?;

        DatabaseItem::update_color_by_id(
            &mut *transaction,
            item_id,
            new_color.0,
            new_color.1,
            new_color.2,
        )
        .await
        .map_err(error::ErrorInternalServerError)?;

        LogEntry::log_action(
            &mut *transaction,
            token.to_string(),
            format!(
                "Changed color of {} #{} from \"{}\" to \"{}\"",
                old_item.r#type, item_id, old_color, new_color
            ),
            None,
            Some(item_id),
        )
        .await
        .map_err(error::ErrorInternalServerError)?;

        updated.push("color")
    }

    if let Some(r#type) = r#type {
        DatabaseItem::update_type_by_id(&mut *transaction, item_id, r#type.into())
            .await
            .map_err(error::ErrorInternalServerError)?;

        LogEntry::log_action(
            &mut *transaction,
            token.to_string(),
            format!(
                "Changed type of item #{} from \"{}\" to \"{}\"",
                item_id,
                old_item.r#type,
                r#type.as_str()
            ),
            None,
            Some(item_id),
        )
        .await
        .map_err(error::ErrorInternalServerError)?;

        updated.push("type")
    }

    transaction
        .commit()
        .instrument(info_span!("Transaction::commit"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let mut changed = updated.join(", ");
    andify_comma_string(&mut changed);

    Ok(HttpResponse::Ok().body(format!(
        "Updated {} for {} {item_id} ({})",
        changed,
        r#type.map_or(&*old_item.r#type, |t| t.as_str()),
        name.as_deref().unwrap_or(&*old_item.name)
    )))
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PatchItemBody {
    pub token: Token,

    #[ts(optional)]
    pub name: Option<String>,
    #[ts(optional)]
    pub short_name: Option<String>,
    #[ts(optional)]
    pub color: Option<String>,
    #[ts(optional)]
    pub r#type: Option<ItemType>,
}
