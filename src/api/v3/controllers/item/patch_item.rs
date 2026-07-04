use crate::api::v3::models::{ItemColor, ItemType};
use crate::models::{ComicId, ItemId, Token};
use crate::util::{andify_comma_string, ensure_is_authorized};
use actix_web::web::Json;
use actix_web::{Result, error, web};
use actix_web_grants::authorities::AuthDetails;
use anyhow::anyhow;
use api_macros::api_endpoint;
use database::DbPool;
use database::models::{Item as DatabaseItem, LogEntry};
use serde::{Deserialize, Deserializer};
use shared::token_permissions;
use tracing::{Instrument, info_span};
use ts_rs::TS;

#[api_endpoint(method = "PATCH", path = "itemdata/{itemId}")]
#[tracing::instrument(skip(pool, auth), fields(permissions = ?auth.authorities))]
#[expect(clippy::too_many_lines)]
pub async fn patch_item(
    pool: web::Data<DbPool>,
    request: web::Json<PatchItemBody>,
    item_id: web::Path<ItemId>,
    auth: AuthDetails,
) -> Result<Json<String>> {
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
        .ok_or_else(|| error::ErrorNotFound(anyhow!("No item with id {item_id} exists")))?;

    let PatchItemBody {
        token,
        name,
        short_name,
        color,
        r#type,
        start_comic_id,
        end_comic_id,
    } = request.into_inner();

    let effective_start_comic_id = start_comic_id
        .map(ComicId::into_inner)
        .or(old_item.start_comic_id);
    let effective_end_comic_id =
        end_comic_id.map_or(old_item.end_comic_id, |end| end.map(ComicId::into_inner));
    if let (Some(start), Some(end)) = (effective_start_comic_id, effective_end_comic_id) {
        if end < start {
            return Err(error::ErrorBadRequest(anyhow!(
                "endComicId ({end}) cannot be before startComicId ({start})"
            )));
        }
    }

    let mut updated = Vec::with_capacity(5);

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

        updated.push("name");
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

        updated.push("short name");
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

        updated.push("color");
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

        updated.push("type");
    }

    if let Some(start_comic_id) = start_comic_id {
        let start_comic_id = start_comic_id.into_inner();
        DatabaseItem::update_start_comic_id_by_id(&mut *transaction, item_id, start_comic_id)
            .await
            .map_err(error::ErrorInternalServerError)?;

        LogEntry::log_action(
            &mut *transaction,
            token.to_string(),
            format!("Changed startComicId of item #{item_id} to {start_comic_id}"),
            None,
            Some(item_id),
        )
        .await
        .map_err(error::ErrorInternalServerError)?;

        updated.push("start comic");
    }

    if let Some(end_comic_id) = end_comic_id {
        let end_comic_id = end_comic_id.map(ComicId::into_inner);
        DatabaseItem::update_end_comic_id_by_id(&mut *transaction, item_id, end_comic_id)
            .await
            .map_err(error::ErrorInternalServerError)?;

        LogEntry::log_action(
            &mut *transaction,
            token.to_string(),
            end_comic_id.map_or_else(
                || format!("Set item #{item_id} to ongoing (no endComicId)"),
                |end_comic_id| format!("Changed endComicId of item #{item_id} to {end_comic_id}"),
            ),
            None,
            Some(item_id),
        )
        .await
        .map_err(error::ErrorInternalServerError)?;

        updated.push("end comic");
    }

    transaction
        .commit()
        .instrument(info_span!("Transaction::commit"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let mut changed = updated.join(", ");
    andify_comma_string(&mut changed);

    Ok(Json(format!(
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
    /// Only meaningful for storyline items.
    #[ts(optional)]
    pub start_comic_id: Option<ComicId>,
    /// Only meaningful for storyline items. Three states: field omitted
    /// (don't touch), `null` (ongoing), or a value (finished at that comic).
    #[expect(
        clippy::option_option,
        reason = "distinguishes omitted (don't touch) from null (ongoing) from a value"
    )]
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    #[ts(optional)]
    pub end_comic_id: Option<Option<ComicId>>,
}

/// Deserializes a `T | null | undefined` JSON field into `Option<Option<T>>`,
/// distinguishing "field omitted" (`None`, via `#[serde(default)]`) from
/// "field present and null" (`Some(None)`) from "field present with a value"
/// (`Some(Some(value))`) — the classic serde "double option" trick.
#[expect(
    clippy::option_option,
    reason = "double option is the point of this function"
)]
fn deserialize_optional_nullable<'de, D, T>(
    deserializer: D,
) -> std::result::Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}
