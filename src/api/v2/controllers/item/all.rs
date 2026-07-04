use actix_web::{HttpResponse, Result, error, web};
use database::DbPool;
use database::models::Item as DatabaseItem;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use tracing::{Instrument, info_span};

use crate::api::v2::controllers::comic::navigation_data::{
    ItemNavigationDataSorting, fetch_all_item_navigation_data,
};
use crate::api::v2::models::{ItemColor, ItemList, ItemType, UnhydratedItemNavigationData};
use crate::models::{ComicId, ItemId};

#[tracing::instrument(skip(pool))]
pub(crate) async fn all(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let all_items = DatabaseItem::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let all_navigation_items = fetch_all_item_navigation_data(
        &mut conn,
        ComicId::from_trusted(1),
        None,
        None,
        ItemNavigationDataSorting::ByCount,
    )
    .await?
    .into_iter()
    .map(|i| (i.id, i))
    .collect::<BTreeMap<ItemId, UnhydratedItemNavigationData>>();

    let mut items = vec![];
    for item in all_items {
        let DatabaseItem {
            id,
            short_name,
            name,
            r#type,
            color_red,
            color_green,
            color_blue,
            primary_image: _,
            start_comic_id: _,
            end_comic_id: _,
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
            count: i32::try_from(count).unwrap(),
        });
    }

    items.sort_by_key(|i| Reverse(i.count));

    Ok(HttpResponse::Ok().json(items))
}
