use crate::models::Token;
use crate::util::{ensure_is_authorized, read_capped, MAX_IMAGE_BYTES};
use actix_multipart::Multipart;
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::authorities::AuthDetails;
use anyhow::anyhow;
use crc32c::crc32c;
use database::models::{Item as DatabaseItem, LogEntry, Token as DatabaseToken};
use database::DbPool;
use futures::StreamExt;
use shared::token_permissions;
use std::str::FromStr;

pub(crate) async fn image_upload(
    pool: web::Data<DbPool>,
    image_upload_form: Multipart,
) -> Result<HttpResponse> {
    let FormData {
        token,
        item_id,
        image: (image, crc32c_hash),
    } = get_form_data(image_upload_form).await?;
    let token = Token::from(token);

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

    let new_item_image_id =
        i32::try_from(result.last_insert_id()).expect("new image ID fits in i32");

    LogEntry::log_action(
        &mut *transaction,
        token.to_string(),
        format!("Uploaded image #{new_item_image_id} for item #{item_id}"),
        None,
        Some(item_id),
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    transaction
        .commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().finish())
}

struct FormData {
    token: uuid::Uuid,
    item_id: u16,
    image: (Vec<u8>, u32),
}

async fn get_form_data(mut image_upload_form: Multipart) -> Result<FormData> {
    let mut token: Option<uuid::Uuid> = None;
    let mut item_id: Option<u16> = None;
    let mut image: Option<(Vec<u8>, u32)> = None;

    while let Some(item) = image_upload_form.next().await {
        let mut field = item.map_err(error::ErrorBadRequest)?;

        let name = field
            .content_disposition()
            .and_then(|cd| cd.get_name())
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
                let data = read_capped(&mut field, MAX_IMAGE_BYTES).await?;
                let crc32c = crc32c(&data);

                image = Some((data, crc32c));
            }
            _ => {
                return Err(error::ErrorBadRequest(anyhow!(
                    "Encountered unexpected field \"{name}\""
                )));
            }
        }
    }

    let token = token.ok_or_else(|| error::ErrorBadRequest(anyhow!("Missing field \"Token\"")))?;
    let item_id =
        item_id.ok_or_else(|| error::ErrorBadRequest(anyhow!("Missing field \"ItemId\"")))?;
    let image = image.ok_or_else(|| error::ErrorBadRequest(anyhow!("Missing field \"Image\"")))?;

    Ok(FormData {
        token,
        item_id,
        image,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::test_support::multipart_from_fields;
    use actix_web::http::StatusCode;

    const TOKEN: &[u8] = b"00000000-0000-0000-0000-000000000000";

    #[tokio::test]
    async fn accepts_image_at_the_size_limit() {
        let image = vec![0xAB; MAX_IMAGE_BYTES];
        let form = multipart_from_fields(&[("Token", TOKEN), ("ItemId", b"7"), ("Image", &image)]);

        let data = get_form_data(form).await.unwrap();

        assert_eq!(data.item_id, 7);
        assert_eq!(data.image.0.len(), MAX_IMAGE_BYTES);
    }

    #[tokio::test]
    async fn rejects_image_over_the_size_limit_with_payload_too_large() {
        let image = vec![0xAB; MAX_IMAGE_BYTES + 1];
        let form = multipart_from_fields(&[("Token", TOKEN), ("ItemId", b"7"), ("Image", &image)]);

        let Err(err) = get_form_data(form).await else {
            panic!("oversized image was accepted");
        };

        assert_eq!(
            err.as_response_error().status_code(),
            StatusCode::PAYLOAD_TOO_LARGE
        );
    }
}
