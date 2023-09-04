use crate::models::v2::{ImageId, ItemImageList, Token};
use crate::util::ensure_is_authorized;
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::AuthDetails;
use anyhow::anyhow;
use database::models::{Item as DatabaseItem, LogEntry};
use database::DbPool;
use serde::Deserialize;
use shared::token_permissions;
use ts_rs::TS;

pub(crate) async fn images(
    pool: web::Data<DbPool>,
    item_id: web::Path<u16>,
) -> Result<HttpResponse> {
    let item_image_list =
        DatabaseItem::image_metadatas_by_id_with_mapping(&***pool, *item_id, ItemImageList::from)
            .await
            .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(item_image_list))
}

pub(crate) async fn image(
    pool: web::Data<DbPool>,
    image_id: web::Path<u32>,
) -> Result<HttpResponse> {
    let image = DatabaseItem::image_by_image_id(&***pool, *image_id)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| {
            error::ErrorNotFound(anyhow!("No item image with id {} exists", *image_id))
        })?;

    Ok(HttpResponse::Ok().content_type("image/png").body(image))
}

pub(crate) async fn delete(
    pool: web::Data<DbPool>,
    image_id: web::Path<u32>,
    request: web::Json<DeleteImageBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    ensure_is_authorized(&auth, token_permissions::CAN_REMOVE_IMAGE_FROM_ITEM)
        .map_err(error::ErrorForbidden)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let image_id = image_id.into_inner();

    let item_id = DatabaseItem::item_id_by_image_id(&mut *transaction, image_id)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| {
            error::ErrorNotFound(anyhow!("No item image with id {} exists", image_id))
        })?;

    let result = DatabaseItem::delete_image(&mut *transaction, image_id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    if result.rows_affected() != 1 {
        return Err(error::ErrorNotFound(format!(
            "Could not delete image {}; the image did not exist",
            image_id
        )));
    }

    LogEntry::log_action(
        &mut *transaction,
        request.token.to_string(),
        format!("Deleted image #{}", image_id),
        None,
        Some(item_id),
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    transaction
        .commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(format!("Deleted image #{}", image_id)))
}

pub(crate) async fn set_primary(
    pool: web::Data<DbPool>,
    item_id: web::Path<u16>,
    request: web::Json<SetPrimaryImageBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    ensure_is_authorized(&auth, token_permissions::CAN_CHANGE_ITEM_DATA)
        .map_err(error::ErrorForbidden)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let item_id = item_id.into_inner();

    let result =
        DatabaseItem::set_primary_image(&mut *transaction, item_id, request.image_id.into_inner())
            .await
            .map_err(error::ErrorInternalServerError)?;

    if result.rows_affected() != 1 {
        return Err(error::ErrorNotFound(format!(
            "Could not set image #{} as primary for item #{}",
            request.image_id, item_id
        )));
    }

    LogEntry::log_action(
        &mut *transaction,
        request.token.to_string(),
        format!(
            "Set image #{} as primary for item #{}",
            request.image_id, item_id
        ),
        None,
        Some(item_id),
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    transaction
        .commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(format!("Primary image changed for item {item_id}")))
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub(crate) struct DeleteImageBody {
    token: Token,
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub(crate) struct SetPrimaryImageBody {
    token: Token,
    image_id: ImageId,
}
