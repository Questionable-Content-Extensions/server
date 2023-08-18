use crate::models::v1::Token;
use crate::util::ensure_is_authorized;
use actix_multipart::Multipart;
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::AuthDetails;
use anyhow::anyhow;
use crc32c::crc32c;
use database::models::{Item as DatabaseItem, LogEntry, Token as DatabaseToken};
use database::DbPool;
use futures::StreamExt;
use shared::token_permissions;
use std::str::FromStr;

#[allow(clippy::too_many_lines)]
pub(crate) async fn image_upload(
    pool: web::Data<DbPool>,
    mut image_upload_form: Multipart,
) -> Result<HttpResponse> {
    let mut token: Option<uuid::Uuid> = None;
    let mut item_id: Option<u16> = None;
    let mut image: Option<(Vec<u8>, u32)> = None;

    while let Some(item) = image_upload_form.next().await {
        let mut field = item.map_err(error::ErrorBadRequest)?;

        let content_disposition = field.content_disposition();
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
