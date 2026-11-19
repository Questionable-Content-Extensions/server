use crate::api::v1::models::{Item, ItemColor, ItemImageList, ItemType, RelatedItem};
use crate::models::ItemId;
use actix_web::{HttpResponse, Result, error, web};
use anyhow::anyhow;
use database::DbPool;
use database::models::{Item as DatabaseItem, RelatedItem as RelatedDatabaseItem};
use std::convert::{TryFrom, TryInto};

#[expect(
    clippy::cast_precision_loss,
    reason = "comic/item counts are well within f64 mantissa precision"
)]
pub(crate) async fn by_id(
    pool: web::Data<DbPool>,
    item_id: web::Path<ItemId>,
) -> Result<HttpResponse> {
    let item_id = item_id.into_inner();

    let mut conn = pool
        .acquire()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let item = DatabaseItem::by_id(&mut *conn, item_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound(anyhow!("No item with id {item_id} exists")))?;

    let stats = DatabaseItem::occurrence_stats_by_id(&mut *conn, item_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?;

    let item = Item {
        id: item_id,
        short_name: item.short_name,
        name: item.name,
        r#type: ItemType::try_from(&*item.r#type).map_err(error::ErrorInternalServerError)?,
        color: ItemColor(item.color_red, item.color_green, item.color_blue),
        first: stats
            .first
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds")
            .unwrap_or_default(),
        last: stats
            .last
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds")
            .unwrap_or_default(),
        appearances: i32::try_from(stats.count).unwrap(),
        total_comics: i32::try_from(stats.total_comics).unwrap(),
        presence: if stats.total_comics == 0 {
            0.0
        } else {
            stats.count as f64 * 100.0 / stats.total_comics as f64
        },
        has_image: stats.image_count > 0,
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
            let id = id.into();

            RelatedItem {
                id,
                short_name,
                name,
                r#type: ItemType::try_from(&*r#type).expect("Item types in the database are valid"),
                color: ItemColor(color_red, color_green, color_blue),
                count: i32::try_from(count).unwrap(),
            }
        },
    )
    .await
    .map_err(error::ErrorInternalServerError)
}
