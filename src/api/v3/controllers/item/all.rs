use crate::api::v3::models::{ItemColor, ItemList, ItemType};
use actix_web::web::Json;
use actix_web::{Result, error, web};
use api_macros::api_endpoint;
use database::DbPool;
use database::models::Item as DatabaseItem;
use std::convert::{TryFrom, TryInto};
use tracing::{Instrument, info_span};

#[api_endpoint(method = "GET", path = "itemdata/")]
#[tracing::instrument(skip(pool))]
pub async fn all(pool: web::Data<DbPool>) -> Result<Json<Vec<ItemList>>> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let all_items = DatabaseItem::all_with_counts(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let mut items = Vec::with_capacity(all_items.len());
    for item in all_items {
        items.push(ItemList {
            id: item.id.into(),
            short_name: item.short_name,
            name: item.name,
            r#type: ItemType::try_from(&*item.r#type).map_err(error::ErrorInternalServerError)?,
            color: ItemColor(item.color_red, item.color_green, item.color_blue),
            count: i32::try_from(item.count).expect("known to be much smaller than i32::MAX"),
            start_comic_id: item
                .start_comic_id
                .map(TryInto::try_into)
                .transpose()
                .expect("database has valid comicIds"),
            end_comic_id: item
                .end_comic_id
                .map(TryInto::try_into)
                .transpose()
                .expect("database has valid comicIds"),
        });
    }

    Ok(Json(items))
}
