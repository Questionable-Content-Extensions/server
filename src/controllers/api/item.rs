use crate::models::{ItemColor, Token};
use crate::util::ensure_is_authorized;
use actix_multipart::Multipart;
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::AuthDetails;
use anyhow::anyhow;
use crc32c::crc32c;
use database::models::{Item as DatabaseItem, LogEntry, Token as DatabaseToken};
use database::DbPool;
use futures::StreamExt;
use serde::Deserialize;
use shared::token_permissions;
use std::str::FromStr;

mod all;
mod by_id;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(all::all)))
        .service(web::resource("setproperty").route(web::post().to(set_property)))
        .service(web::resource("image/upload").route(web::post().to(image_upload)))
        .service(web::resource("image/{imageId}").route(web::get().to(by_id::image)))
        .service(web::resource("friends/{itemId}").route(web::get().to(by_id::friends)))
        .service(web::resource("locations/{itemId}").route(web::get().to(by_id::locations)))
        .service(web::resource("{itemId}").route(web::get().to(by_id::by_id)))
        .service(web::resource("{itemId}/friends").route(web::get().to(by_id::friends)))
        .service(web::resource("{itemId}/locations").route(web::get().to(by_id::locations)))
        .service(web::resource("{itemId}/images").route(web::get().to(by_id::images)));
}

#[allow(clippy::too_many_lines)]
async fn image_upload(
    pool: web::Data<DbPool>,
    mut image_upload_form: Multipart,
) -> Result<HttpResponse> {
    let mut token: Option<uuid::Uuid> = None;
    let mut item_id: Option<u16> = None;
    let mut image: Option<(Vec<u8>, u32)> = None;

    while let Some(item) = image_upload_form.next().await {
        let mut field = item.map_err(error::ErrorInternalServerError)?;

        let content_disposition = field.content_disposition().ok_or_else(|| {
            error::ErrorBadRequest(anyhow!("Content-Disposition header was missing"))
        })?;
        let name = content_disposition
            .get_name()
            .ok_or_else(|| error::ErrorBadRequest(anyhow!("A form field was missing a name")))?;

        match name {
            "Token" => {
                let data = field
                    .next()
                    .await
                    .ok_or_else(|| {
                        error::ErrorBadRequest(anyhow!("Token form field was missing a value"))
                    })?
                    .map_err(error::ErrorBadRequest)?;
                let value = uuid::Uuid::from_str(
                    std::str::from_utf8(&data).map_err(error::ErrorBadRequest)?,
                )
                .map_err(error::ErrorBadRequest)?;

                token = Some(value);
            }
            "ItemId" => {
                let data = field
                    .next()
                    .await
                    .ok_or_else(|| {
                        error::ErrorBadRequest(anyhow!("ItemId form field was missing a value"))
                    })?
                    .map_err(error::ErrorBadRequest)?;
                let value: u16 = std::str::from_utf8(&data)
                    .map_err(error::ErrorBadRequest)?
                    .parse()
                    .map_err(error::ErrorBadRequest)?;

                item_id = Some(value);
            }
            "Image" => {
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    data.extend_from_slice(&chunk?[..]);
                }
                let crc32c = crc32c(&data);

                image = Some((data, crc32c));
            }
            _ => {
                return Err(error::ErrorBadRequest(anyhow!(
                    "Encountered unexpected field \"{}\"",
                    name
                )))
            }
        }
    }

    let token = Token::from(
        token.ok_or_else(|| error::ErrorBadRequest(anyhow!("Missing field \"Token\"")))?,
    );
    let item_id =
        item_id.ok_or_else(|| error::ErrorBadRequest(anyhow!("Missing field \"ItemId\"")))?;
    let (image, crc32c_hash) =
        image.ok_or_else(|| error::ErrorBadRequest(anyhow!("Missing field \"Image\"")))?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let permissions =
        DatabaseToken::get_permissions_for_token(&mut *transaction, token.to_string())
            .await
            .map_err(error::ErrorInternalServerError)?;
    let auth = AuthDetails::new(permissions);

    ensure_is_authorized(&auth, token_permissions::CAN_ADD_IMAGE_TO_ITEM)
        .map_err(error::ErrorForbidden)?;

    let result = DatabaseItem::create_image(&mut *transaction, item_id, image, crc32c_hash)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let new_item_image_id = result.last_insert_id() as i32;

    LogEntry::log_action(
        &mut *transaction,
        token.to_string(),
        format!(
            "Uploaded image #{} for item #{}",
            new_item_image_id, item_id
        ),
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    transaction
        .commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().finish())
}

#[allow(clippy::too_many_lines)]
async fn set_property(
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
struct SetItemPropertyBody {
    token: Token,
    #[serde(rename = "item")]
    item_id: u16,
    property: String,
    value: String,
}
