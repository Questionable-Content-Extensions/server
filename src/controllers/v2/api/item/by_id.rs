use crate::models::v1::{ItemColor, ItemId, ItemType};
use crate::models::v2::{Item, RelatedItem};
use actix_web::{error, web, HttpResponse, Result};
use anyhow::anyhow;
use database::models::{
    Comic as DatabaseComic, Item as DatabaseItem, RelatedItem as RelatedDatabaseItem,
};
use database::DbPool;
use std::convert::{TryFrom, TryInto};

pub(crate) async fn by_id(
    pool: web::Data<DbPool>,
    item_id: web::Path<ItemId>,
) -> Result<HttpResponse> {
    let item_id = item_id.into_inner();

    let mut conn = pool
        .acquire()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let item = DatabaseItem::by_id(&mut conn, item_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound(anyhow!("No item with id {} exists", item_id)))?;

    let primary_image = if let Some(primary_image) = item.primary_image {
        Some(primary_image)
    } else {
        let image_metadatas = DatabaseItem::image_metadatas_by_id(&***pool, item_id.into_inner())
            .await
            .map_err(error::ErrorInternalServerError)?;

        image_metadatas.first().map(|first_image| first_image.id)
    };

    let item_occurrence =
        DatabaseItem::first_and_last_apperance_and_count_by_id(&mut conn, item_id.into_inner())
            .await
            .map_err(error::ErrorInternalServerError)?;

    let total_comics = DatabaseComic::count(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let image_count = DatabaseItem::image_count_by_id(&mut conn, item_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?;

    let item = Item {
        id: item_id,
        short_name: item.short_name,
        name: item.name,
        r#type: ItemType::try_from(&*item.r#type).map_err(error::ErrorInternalServerError)?,
        color: ItemColor(item.color_red, item.color_green, item.color_blue),
        first: item_occurrence
            .first
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds")
            .unwrap_or_default(),
        last: item_occurrence
            .last
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds")
            .unwrap_or_default(),
        appearances: i32::try_from(item_occurrence.count).unwrap(),
        total_comics: i32::try_from(total_comics).unwrap(),
        presence: if total_comics == 0 {
            0.0
        } else {
            item_occurrence.count as f64 * 100.0 / total_comics as f64
        },
        has_image: image_count > 0,
        primary_image,
    };

    Ok(HttpResponse::Ok().json(item))
}

pub(crate) async fn friends(
    pool: web::Data<DbPool>,
    item_id: web::Path<u16>,
) -> Result<HttpResponse> {
    let items = related_items(pool, *item_id, ItemType::Cast, 5).await?;

    Ok(HttpResponse::Ok().json(items))
}

pub(crate) async fn locations(
    pool: web::Data<DbPool>,
    item_id: web::Path<u16>,
) -> Result<HttpResponse> {
    let items = related_items(pool, *item_id, ItemType::Location, 5).await?;

    Ok(HttpResponse::Ok().json(items))
}

async fn related_items(
    pool: web::Data<DbPool>,
    item_id: u16,
    r#type: ItemType,
    amount: i64,
) -> Result<Vec<RelatedItem>> {
    DatabaseItem::related_items_by_id_and_type_with_mapping(
        &***pool,
        item_id,
        r#type.into(),
        amount,
        |ri| {
            let RelatedDatabaseItem { id, count, .. } = ri;
            let id = id.into();

            RelatedItem {
                id,
                count: i32::try_from(count).unwrap(),
            }
        },
    )
    .await
    .map_err(error::ErrorInternalServerError)
}
