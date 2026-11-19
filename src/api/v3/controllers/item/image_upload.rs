use crate::models::Token;
use crate::util::{MAX_IMAGE_BYTES, ensure_is_authorized, read_capped};
use actix_multipart::Multipart;
use actix_web::{HttpResponse, Result, error, web};
use actix_web_grants::authorities::AuthDetails;
use anyhow::anyhow;
use crc32c::crc32c;
use database::DbPool;
use database::models::{Item as DatabaseItem, LogEntry};
use futures::StreamExt;
use shared::token_permissions;
use tracing::{Instrument, info_span};

#[tracing::instrument(skip(pool, image_upload_form, auth), fields(image.size, image.crc32c, permissions = ?auth.authorities))]
pub async fn image_upload(
    pool: web::Data<DbPool>,
    item_id: web::Path<u16>,
    image_upload_form: Multipart,
    token: web::ReqData<Token>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    ensure_is_authorized(&auth, token_permissions::CAN_ADD_IMAGE_TO_ITEM)
        .map_err(error::ErrorForbidden)?;

    let token = *token;
    let FormData { image } = get_form_data(image_upload_form).await?;

    tracing::Span::current()
        .record("image.size", image.0.len())
        .record("image.crc32c", image.1);

    let item_id = item_id.into_inner();
    let (image, crc32c_hash) = image;

    let mut transaction = pool
        .begin()
        .instrument(info_span!("Pool::begin"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let result = DatabaseItem::create_image(&mut *transaction, item_id, image, crc32c_hash)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let new_item_image_id =
        i32::try_from(result.last_insert_id()).expect("new image ID fits in i32");

    let action = format!("Uploaded image #{new_item_image_id} for item #{item_id}");
    LogEntry::log_action(
        &mut *transaction,
        token.to_string(),
        &action,
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

    Ok(HttpResponse::Ok().body(action))
}

struct FormData {
    image: (Vec<u8>, u32),
}
async fn get_form_data(mut image_upload_form: Multipart) -> Result<FormData, actix_web::Error> {
    let mut image: Option<(Vec<u8>, u32)> = None;
    while let Some(item) = image_upload_form.next().await {
        let mut field = item.map_err(error::ErrorBadRequest)?;

        let name = field
            .content_disposition()
            .and_then(|cd| cd.get_name())
            .ok_or_else(|| error::ErrorBadRequest(anyhow!("A form field was missing a name")))?;

        match name {
            "image" => {
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

    let image = image.ok_or_else(|| error::ErrorBadRequest(anyhow!("Missing field \"image\"")))?;
    Ok(FormData { image })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::test_support::multipart_from_fields;
    use actix_web::http::StatusCode;

    #[tokio::test]
    async fn accepts_image_at_the_size_limit() {
        let image = vec![0xAB; MAX_IMAGE_BYTES];
        let form = multipart_from_fields(&[("image", &image)]);

        let data = get_form_data(form).await.unwrap();

        assert_eq!(data.image.0.len(), MAX_IMAGE_BYTES);
    }

    #[tokio::test]
    async fn rejects_image_over_the_size_limit_with_payload_too_large() {
        let image = vec![0xAB; MAX_IMAGE_BYTES + 1];
        let form = multipart_from_fields(&[("image", &image)]);

        let Err(err) = get_form_data(form).await else {
            panic!("oversized image was accepted");
        };

        assert_eq!(
            err.as_response_error().status_code(),
            StatusCode::PAYLOAD_TOO_LARGE
        );
    }
}
