use crate::controllers::api::comic::navigation_data::fetch_all_item_navigation_data;
use crate::database::models::Item as DatabaseItem;
use crate::database::DbPool;
use crate::models::{
    Item, ItemColor, ItemImageList, ItemList, ItemNavigationData, ItemType, RelatedItem,
};
use actix_web::{error, web, HttpResponse, Result};
use anyhow::anyhow;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::convert::TryFrom;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(all)))
        .service(web::resource("/{itemId}").route(web::get().to(by_id)))
        .service(web::resource("/friends/{itemId}").route(web::get().to(friends)))
        .service(web::resource("{itemId}/friends").route(web::get().to(friends)))
        .service(web::resource("/locations/{itemId}").route(web::get().to(locations)))
        .service(web::resource("{itemId}/locations").route(web::get().to(locations)))
        .service(web::resource("{itemId}/images").route(web::get().to(images)))
        .service(web::resource("image/{imageId}").route(web::get().to(image)));
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
            color: ItemColor::new(color_red, color_green, color_blue),
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
        color: ItemColor::new(item.Color_Red, item.Color_Green, item.Color_Blue),
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
            color: ItemColor::new(color_red, color_green, color_blue),
            count,
        };
        items.push(item);
    }

    Ok(items)
}
