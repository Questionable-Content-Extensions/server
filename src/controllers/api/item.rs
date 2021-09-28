use crate::controllers::api::comic::navigation_data::fetch_all_item_navigation_data;
use crate::database::models::Item as DatabaseItem;
use crate::database::DbPool;
use crate::models::{
    token_permissions, Item, ItemColor, ItemImageList, ItemList, ItemNavigationData, ItemType,
    RelatedItem,
};
use crate::util::{ensure_is_authorized, get_permissions_for_token, log_action};
use actix_multipart::Multipart;
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::AuthDetails;
use anyhow::anyhow;
use crc32c::crc32c;
use futures::StreamExt;
use serde::Deserialize;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::str::FromStr;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(all)))
        .service(web::resource("setproperty").route(web::post().to(set_property)))
        .service(web::resource("image/upload").route(web::post().to(image_upload)))
        .service(web::resource("image/{imageId}").route(web::get().to(image)))
        .service(web::resource("friends/{itemId}").route(web::get().to(friends)))
        .service(web::resource("locations/{itemId}").route(web::get().to(locations)))
        .service(web::resource("{itemId}").route(web::get().to(by_id)))
        .service(web::resource("{itemId}/friends").route(web::get().to(friends)))
        .service(web::resource("{itemId}/locations").route(web::get().to(locations)))
        .service(web::resource("{itemId}/images").route(web::get().to(images)));
}

async fn all(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let all_items = sqlx::query_as!(
        DatabaseItem,
        r#"
				SELECT *
				FROM items
			"#,
    )
    .fetch_all(&mut conn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    let all_navigation_items = fetch_all_item_navigation_data(&mut conn, 1, None, None)
        .await?
        .into_iter()
        .map(|i| (i.id, i))
        .collect::<BTreeMap<i16, ItemNavigationData>>();

    let mut items = vec![];
    for item in all_items {
        let DatabaseItem {
            id,
            shortName: short_name,
            name,
            r#type,
            Color_Red: color_red,
            Color_Green: color_green,
            Color_Blue: color_blue,
        } = item;

        let count = all_navigation_items
            .get(&item.id)
            .map(|i| i.count)
            .unwrap_or_default();

        items.push(ItemList {
            id,
            short_name,
            name,
            r#type: ItemType::try_from(&*r#type).map_err(error::ErrorInternalServerError)?,
            color: ItemColor(color_red, color_green, color_blue),
            count,
        })
    }

    items.sort_by_key(|i| Reverse(i.count));

    Ok(HttpResponse::Ok().json(items))
}

async fn by_id(pool: web::Data<DbPool>, item_id: web::Path<i16>) -> Result<HttpResponse> {
    struct Occurrence {
        min: Option<i16>,
        max: Option<i16>,
        count: i64,
    }
    let mut conn = pool
        .acquire()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let item = sqlx::query_as!(
        DatabaseItem,
        r#"
            SELECT * FROM `items`
            WHERE `id` = ?
        "#,
        *item_id
    )
    .fetch_optional(&mut conn)
    .await
    .map_err(error::ErrorInternalServerError)?
    .ok_or_else(|| error::ErrorNotFound(anyhow!("No item with id {} exists", *item_id)))?;

    let item_occurrence = sqlx::query_as!(
        Occurrence,
        r#"
            SELECT
                MIN(comic_id) AS min,
                MAX(comic_id) AS max,
                COUNT(comic_id) AS count
            FROM `occurences`
            WHERE `items_id` = ?
        "#,
        *item_id
    )
    .fetch_one(&mut conn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    let total_comics = sqlx::query_scalar!(
        r#"
            SELECT COUNT(*) FROM `comic`
        "#,
    )
    .fetch_one(&mut conn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    let image_count = sqlx::query_scalar!(
        r#"
            SELECT COUNT(*) FROM `ItemImages`
            WHERE ItemId = ?
        "#,
        *item_id
    )
    .fetch_one(&mut conn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    let item = Item {
        id: *item_id,
        short_name: item.shortName,
        name: item.name,
        r#type: ItemType::try_from(&*item.r#type).map_err(error::ErrorInternalServerError)?,
        color: ItemColor(item.Color_Red, item.Color_Green, item.Color_Blue),
        first: item_occurrence.min.unwrap_or_default(),
        last: item_occurrence.max.unwrap_or_default(),
        appearances: item_occurrence.count,
        total_comics,
        presence: if total_comics == 0 {
            0.0
        } else {
            item_occurrence.count as f64 * 100.0 / total_comics as f64
        },
        has_image: image_count > 0,
    };

    Ok(HttpResponse::Ok().json(item))
}

async fn friends(pool: web::Data<DbPool>, item_id: web::Path<i16>) -> Result<HttpResponse> {
    let items = related_items(pool, *item_id, ItemType::Cast, 5).await?;

    Ok(HttpResponse::Ok().json(items))
}

async fn locations(pool: web::Data<DbPool>, item_id: web::Path<i16>) -> Result<HttpResponse> {
    let items = related_items(pool, *item_id, ItemType::Location, 5).await?;

    Ok(HttpResponse::Ok().json(items))
}

async fn images(pool: web::Data<DbPool>, item_id: web::Path<i16>) -> Result<HttpResponse> {
    let item_image_list = sqlx::query_as!(
        ItemImageList,
        r#"
            SELECT
                Id AS id,
                CRC32CHash AS crc32c_hash
            FROM `ItemImages`
            WHERE ItemId = ?
        "#,
        *item_id
    )
    .fetch_all(&***pool)
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(item_image_list))
}

async fn image(pool: web::Data<DbPool>, image_id: web::Path<i32>) -> Result<HttpResponse> {
    let image = sqlx::query_scalar!(
        r#"
            SELECT Image FROM `ItemImages`
            WHERE Id = ?
        "#,
        *image_id
    )
    .fetch_optional(&***pool)
    .await
    .map_err(error::ErrorInternalServerError)?
    .ok_or_else(|| error::ErrorNotFound(anyhow!("No item image with id {} exists", *image_id)))?;

    Ok(HttpResponse::Ok().content_type("image/png").body(image))
}

#[allow(clippy::too_many_lines)]
async fn image_upload(
    pool: web::Data<DbPool>,
    mut image_upload_form: Multipart,
) -> Result<HttpResponse> {
    let mut token: Option<Uuid> = None;
    let mut item_id: Option<i16> = None;
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
                let value =
                    Uuid::from_str(std::str::from_utf8(&data).map_err(error::ErrorBadRequest)?)
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
                let value: i16 = std::str::from_utf8(&data)
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

    let token = token.ok_or_else(|| error::ErrorBadRequest(anyhow!("Missing field \"Token\"")))?;
    let item_id =
        item_id.ok_or_else(|| error::ErrorBadRequest(anyhow!("Missing field \"ItemId\"")))?;
    let (image, crc32c_hash) =
        image.ok_or_else(|| error::ErrorBadRequest(anyhow!("Missing field \"Image\"")))?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let permissions = get_permissions_for_token(&mut *transaction, token)
        .await
        .map_err(error::ErrorInternalServerError)?;
    let auth = AuthDetails::new(permissions);

    ensure_is_authorized(&auth, token_permissions::CAN_ADD_IMAGE_TO_ITEM)
        .map_err(error::ErrorForbidden)?;

    let result = sqlx::query!(
        r#"
            INSERT INTO `ItemImages`
                (ItemId, Image, CRC32CHash)
            VALUES
                (?, ?, ?)
        "#,
        item_id,
        image,
        crc32c_hash,
    )
    .execute(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?;

    let new_item_image_id = result.last_insert_id() as i32;

    log_action(
        &mut *transaction,
        token,
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

    let old_item = sqlx::query_as!(
        DatabaseItem,
        r#"
            SELECT *
            FROM items
            WHERE id = ?
        "#,
        request.item_id
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?;

    match &*request.property {
        "name" => {
            sqlx::query!(
                r#"
                    UPDATE `items`
                    SET name = ?
                    WHERE
                        id = ?
                "#,
                request.value,
                request.item_id,
            )
            .execute(&mut *transaction)
            .await
            .map_err(error::ErrorInternalServerError)?;

            if old_item.name.is_empty() {
                log_action(
                    &mut *transaction,
                    request.token,
                    format!(
                        "Set name of {} #{} to \"{}\"",
                        old_item.r#type, request.item_id, request.value
                    ),
                )
                .await
                .map_err(error::ErrorInternalServerError)?;
            } else {
                log_action(
                    &mut *transaction,
                    request.token,
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
            sqlx::query!(
                r#"
                    UPDATE `items`
                    SET shortName = ?
                    WHERE
                        id = ?
                "#,
                request.value,
                request.item_id,
            )
            .execute(&mut *transaction)
            .await
            .map_err(error::ErrorInternalServerError)?;

            if old_item.shortName.is_empty() {
                log_action(
                    &mut *transaction,
                    request.token,
                    format!(
                        "Set shortName of {} #{} to \"{}\"",
                        old_item.r#type, request.item_id, request.value
                    ),
                )
                .await
                .map_err(error::ErrorInternalServerError)?;
            } else {
                log_action(
                    &mut *transaction,
                    request.token,
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

            sqlx::query!(
                r#"
                    UPDATE `items`
                    SET
                        Color_Red = ?,
                        Color_Green = ?,
                        Color_Blue = ?
                    WHERE
                        id = ?
                "#,
                new_color.0,
                new_color.1,
                new_color.2,
                request.item_id
            )
            .execute(&mut *transaction)
            .await
            .map_err(error::ErrorInternalServerError)?;

            log_action(
                &mut *transaction,
                request.token,
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

async fn related_items(
    pool: web::Data<DbPool>,
    item_id: i16,
    r#type: ItemType,
    amount: i64,
) -> Result<Vec<RelatedItem>> {
    pub struct RelatedDatabaseItem {
        pub id: i16,
        pub short_name: String,
        pub name: String,
        pub r#type: String,
        pub color_red: u8,
        pub color_green: u8,
        pub color_blue: u8,
        pub count: i64,
    }

    let related_items = sqlx::query_as!(
        RelatedDatabaseItem,
        r#"
            SELECT
                i2.id,
                i2.shortName as short_name,
                i2.name,
                i2.type,
                i2.Color_Red as color_red,
                i2.Color_Green as color_green,
                i2.Color_Blue as color_blue,
                COUNT(i2.id) as count
            FROM items i
            JOIN occurences o ON i.id = o.items_id
            JOIN occurences o2 ON o.comic_id = o2.comic_id
            JOIN items i2 ON o2.items_id = i2.id
            WHERE i.id = ?
                AND i2.id <> i.id
                AND i2.type = ?
            GROUP BY i2.id
            ORDER BY count DESC
            LIMIT ?
			"#,
        item_id,
        r#type,
        amount
    )
    .fetch_all(&***pool)
    .await
    .map_err(error::ErrorInternalServerError)?;

    let mut items = Vec::with_capacity(related_items.len());
    for ri in related_items {
        let RelatedDatabaseItem {
            id,
            short_name,
            name,
            r#type,
            color_red,
            color_green,
            color_blue,
            count,
        } = ri;

        let item = RelatedItem {
            id,
            short_name,
            name,
            r#type: ItemType::try_from(&*r#type).map_err(error::ErrorInternalServerError)?,
            color: ItemColor(color_red, color_green, color_blue),
            count,
        };
        items.push(item);
    }

    Ok(items)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetItemPropertyBody {
    token: uuid::Uuid,
    #[serde(rename = "item")]
    item_id: i16,
    property: String,
    value: String,
}
