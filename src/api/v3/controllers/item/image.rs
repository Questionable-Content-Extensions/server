use crate::api::v3::models::ItemImageList;
use crate::models::{ImageId, Token};
use crate::util::ensure_is_authorized;
use actix_web::web::Json;
use actix_web::{HttpResponse, Result, error, web};
use actix_web_grants::authorities::AuthDetails;
use anyhow::anyhow;
use api_macros::api_endpoint;
use database::DbPool;
use database::models::{Item as DatabaseItem, LogEntry};
use serde::Deserialize;
use shared::token_permissions;
use tracing::{Instrument, info_span};
use ts_rs::TS;

fn detect_mime_type(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if bytes.starts_with(b"\xff\xd8\xff") {
        "image/jpeg"
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        "image/gif"
    } else {
        "application/octet-stream"
    }
}

#[api_endpoint(method = "GET", path = "itemdata/{itemId}/images")]
#[tracing::instrument(skip(pool))]
pub async fn images(
    pool: web::Data<DbPool>,
    item_id: web::Path<u16>,
) -> Result<Json<Vec<ItemImageList>>> {
    let item_image_list =
        DatabaseItem::image_metadatas_by_id_with_mapping(&***pool, *item_id, ItemImageList::from)
            .await
            .map_err(error::ErrorInternalServerError)?;

    Ok(Json(item_image_list))
}

#[tracing::instrument(skip(pool))]
pub async fn image(pool: web::Data<DbPool>, image_id: web::Path<u32>) -> Result<HttpResponse> {
    let image = DatabaseItem::image_by_image_id(&***pool, *image_id)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| {
            error::ErrorNotFound(anyhow!("No item image with id {} exists", *image_id))
        })?;

    let content_type = detect_mime_type(&image);
    Ok(HttpResponse::Ok().content_type(content_type).body(image))
}

#[api_endpoint(method = "DELETE", path = "itemdata/image/{imageId}")]
#[tracing::instrument(skip(pool, auth), fields(permissions = ?auth.authorities))]
pub async fn delete(
    pool: web::Data<DbPool>,
    image_id: web::Path<u32>,
    request: web::Json<DeleteImageBody>,
    auth: AuthDetails,
) -> Result<Json<String>> {
    ensure_is_authorized(&auth, token_permissions::CAN_REMOVE_IMAGE_FROM_ITEM)
        .map_err(error::ErrorForbidden)?;

    let mut transaction = pool
        .begin()
        .instrument(info_span!("Pool::begin"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let image_id = image_id.into_inner();

    let item_id = DatabaseItem::item_id_by_image_id(&mut *transaction, image_id)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound(anyhow!("No item image with id {image_id} exists")))?;

    let result = DatabaseItem::delete_image(&mut *transaction, image_id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    if result.rows_affected() != 1 {
        return Err(error::ErrorNotFound(format!(
            "Could not delete image {image_id}; the image did not exist"
        )));
    }

    LogEntry::log_action(
        &mut *transaction,
        request.token.to_string(),
        format!("Deleted image #{image_id}"),
        None,
        Some(item_id),
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    transaction
        .commit()
        .instrument(info_span!("Transaction::commit"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(Json(format!("Deleted image #{image_id}")))
}

#[api_endpoint(method = "POST", path = "itemdata/{itemId}/images/primary")]
pub async fn set_primary(
    pool: web::Data<DbPool>,
    item_id: web::Path<u16>,
    request: web::Json<SetPrimaryImageBody>,
    auth: AuthDetails,
) -> Result<Json<String>> {
    ensure_is_authorized(&auth, token_permissions::CAN_CHANGE_ITEM_DATA)
        .map_err(error::ErrorForbidden)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let item_id = item_id.into_inner();
    let image_id = request.image_id.into_inner();

    let owner_item_id = DatabaseItem::item_id_by_image_id(&mut *transaction, image_id)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound(anyhow!("No item image with id {image_id} exists")))?;

    if owner_item_id != item_id {
        return Err(error::ErrorBadRequest(anyhow!(
            "Image #{image_id} does not belong to item #{item_id}"
        )));
    }

    let result = DatabaseItem::set_primary_image(&mut *transaction, item_id, image_id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    if result.rows_affected() != 1 {
        return Err(error::ErrorNotFound(format!(
            "Could not set image #{image_id} as primary for item #{item_id}"
        )));
    }

    LogEntry::log_action(
        &mut *transaction,
        request.token.to_string(),
        format!("Set image #{image_id} as primary for item #{item_id}"),
        None,
        Some(item_id),
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    transaction
        .commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(Json(format!("Primary image changed for item {item_id}")))
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeleteImageBody {
    token: Token,
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetPrimaryImageBody {
    token: Token,
    image_id: ImageId,
}
