use crate::models::Token;
use crate::util::{MAX_IMAGE_BYTES, ensure_is_authorized, read_capped};
use actix_multipart::Multipart;
use actix_web::{HttpResponse, Result, error, web};
use actix_web_grants::authorities::AuthDetails;
use anyhow::anyhow;
use crc32c::crc32c;
use database::DbPool;
use database::models::{Item as DatabaseItem, LogEntry, Token as DatabaseToken};
use futures::StreamExt;
use shared::token_permissions;
use std::str::FromStr;
use tracing::{Instrument, info_span};

#[tracing::instrument(skip(pool, image_upload_form),fields(image.size,image.crc32c,permissions))]
pub(crate) async fn image_upload(
    pool: web::Data<DbPool>,
    item_id: web::Path<u16>,
    image_upload_form: Multipart,
) -> Result<HttpResponse> {
    let FormData { token, image } = get_form_data(image_upload_form).await?;

    tracing::Span::current()
        .record("image.size", image.0.len())
        .record("image.crc32c", image.1);

    let token = Token::from(token);
    let item_id = item_id.into_inner();
    let (image, crc32c_hash) = image;

    let mut transaction = pool
        .begin()
        .instrument(info_span!("Pool::begin"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let permissions =
        DatabaseToken::get_permissions_for_token(&mut *transaction, token.to_string())
            .await
            .map_err(error::ErrorInternalServerError)?;
    let auth = AuthDetails::new(permissions);

    tracing::Span::current().record("permissions", format!("{:?}", auth.authorities));

    ensure_is_authorized(&auth, token_permissions::CAN_ADD_IMAGE_TO_ITEM)
        .map_err(error::ErrorForbidden)?;

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
    token: uuid::Uuid,
    image: (Vec<u8>, u32),
}
async fn get_form_data(mut image_upload_form: Multipart) -> Result<FormData, actix_web::Error> {
    let mut token: Option<uuid::Uuid> = None;
    let mut image: Option<(Vec<u8>, u32)> = None;
    while let Some(item) = image_upload_form.next().await {
        let mut field = item.map_err(error::ErrorBadRequest)?;

        let name = field
            .content_disposition()
            .and_then(|cd| cd.get_name())
            .ok_or_else(|| error::ErrorBadRequest(anyhow!("A form field was missing a name")))?;

        match name {
            "token" => {
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

    let token = token.ok_or_else(|| error::ErrorBadRequest(anyhow!("Missing field \"token\"")))?;
    let image = image.ok_or_else(|| error::ErrorBadRequest(anyhow!("Missing field \"Image\"")))?;
    Ok(FormData { token, image })
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
        let form = multipart_from_fields(&[("token", TOKEN), ("image", &image)]);

        let data = get_form_data(form).await.unwrap();

        assert_eq!(data.image.0.len(), MAX_IMAGE_BYTES);
    }

    #[tokio::test]
    async fn rejects_image_over_the_size_limit_with_payload_too_large() {
        let image = vec![0xAB; MAX_IMAGE_BYTES + 1];
        let form = multipart_from_fields(&[("token", TOKEN), ("image", &image)]);

        let Err(err) = get_form_data(form).await else {
            panic!("oversized image was accepted");
        };

        assert_eq!(
            err.as_response_error().status_code(),
            StatusCode::PAYLOAD_TOO_LARGE
        );
    }

    #[test]
    fn handler_does_not_record_the_token_on_its_span() {
        let source = include_str!("image_upload.rs");
        let handler = &source[..source.find("#[cfg(test)]").unwrap()];

        let instrument = handler
            .lines()
            .find(|line| line.starts_with("#[tracing::instrument"))
            .unwrap();
        assert!(
            !instrument.contains("token"),
            "the span declares a token field: {instrument}"
        );
        assert!(
            !handler.contains(r#".record("token""#),
            "the handler records the token on its span"
        );
    }
}
