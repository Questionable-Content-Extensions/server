use crate::controllers::api::comic::navigation_data::fetch_all_item_navigation_data;
use crate::models::{ComicId, ItemColor, ItemId, ItemList, ItemNavigationData, ItemType};
use actix_web::{error, web, HttpResponse, Result};
use database::models::Item as DatabaseItem;
use database::DbPool;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::convert::TryFrom;

pub(crate) async fn all(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let all_items = DatabaseItem::all(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let all_navigation_items =
        fetch_all_item_navigation_data(&mut conn, ComicId::from(1), None, None)
            .await?
            .into_iter()
            .map(|i| (i.id, i))
            .collect::<BTreeMap<ItemId, ItemNavigationData>>();

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
        let id = id.into();

        let count = all_navigation_items
            .get(&id)
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