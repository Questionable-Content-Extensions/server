use crate::api::v3::models::{ComicList, Exclusion, Item, ItemColor, ItemType, RelatedItem};
use crate::models::ItemId;
use actix_web::web::Json;
use actix_web::{Result, error, web};
use anyhow::anyhow;
use api_macros::api_endpoint;
use database::DbPool;
use database::models::{
    Comic as DatabaseComic, Item as DatabaseItem, ItemWithStats, RelatedItem as RelatedDatabaseItem,
};
use serde::Deserialize;
use std::convert::{TryFrom, TryInto};
use tracing::{Instrument, info_span};
use ts_rs::TS;

#[api_endpoint(method = "GET", path = "itemdata/{itemId}")]
#[tracing::instrument(skip(pool))]
pub async fn by_id(pool: web::Data<DbPool>, item_id: web::Path<ItemId>) -> Result<Json<Item>> {
    let item_id = item_id.into_inner();

    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let row = DatabaseItem::by_id_with_stats(&mut *conn, item_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound(anyhow!("No item with id {item_id} exists")))?;

    let item = build_item_response(item_id, row).map_err(error::ErrorInternalServerError)?;

    Ok(Json(item))
}

#[expect(
    clippy::cast_precision_loss,
    reason = "comic/item counts are well within f64 mantissa precision"
)]
fn build_item_response(item_id: ItemId, row: ItemWithStats) -> anyhow::Result<Item> {
    Ok(Item {
        id: item_id,
        short_name: row.short_name,
        name: row.name,
        r#type: ItemType::try_from(&*row.r#type)?,
        color: ItemColor(row.color_red, row.color_green, row.color_blue),
        first: row
            .first
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds")
            .unwrap_or_default(),
        last: row
            .last
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds")
            .unwrap_or_default(),
        appearances: i32::try_from(row.count).expect("known to be much smaller than i32::MAX"),
        total_comics: i32::try_from(row.total_comics)
            .expect("known to be much smaller than i32::MAX"),
        presence: if row.total_comics == 0 {
            0.0
        } else {
            row.count as f64 * 100.0 / row.total_comics as f64
        },
        has_image: row.image_count > 0,
        primary_image: row.primary_image.or(row.first_image_id),
        start_comic_id: row
            .start_comic_id
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds"),
        end_comic_id: row
            .end_comic_id
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds"),
    })
}

#[api_endpoint(method = "GET", path = "itemdata/{itemId}/friends")]
#[tracing::instrument(skip(pool))]
pub async fn friends(
    pool: web::Data<DbPool>,
    item_id: web::Path<u16>,
) -> Result<Json<Vec<RelatedItem>>> {
    let items = related_items(pool, *item_id, ItemType::Cast, 5).await?;

    Ok(Json(items))
}

#[api_endpoint(method = "GET", path = "itemdata/{itemId}/locations")]
#[tracing::instrument(skip(pool))]
pub async fn locations(
    pool: web::Data<DbPool>,
    item_id: web::Path<u16>,
) -> Result<Json<Vec<RelatedItem>>> {
    let items = related_items(pool, *item_id, ItemType::Location, 5).await?;

    Ok(Json(items))
}

#[tracing::instrument(skip(pool))]
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

#[api_endpoint(method = "GET", path = "itemdata/{itemId}/comics")]
#[tracing::instrument(skip(pool))]
pub async fn comics(
    pool: web::Data<DbPool>,
    item_id: web::Path<u16>,
) -> Result<Json<Vec<ComicList>>> {
    let item_id = item_id.into_inner();

    let comics: Vec<ComicList> =
        DatabaseComic::all_with_item_id_mapped(&***pool, item_id, From::from)
            .await
            .map_err(error::ErrorInternalServerError)?;

    Ok(Json(comics))
}

const fn exclusion_to_filter_options(exclude: Option<Exclusion>) -> (Option<bool>, Option<bool>) {
    match exclude {
        None => (None, None),
        Some(Exclusion::Guest) => (Some(false), None),
        Some(Exclusion::NonCanon) => (None, Some(false)),
    }
}

#[api_endpoint(method = "GET", path = "itemdata/{itemId}/comics/random")]
#[tracing::instrument(skip(pool))]
pub async fn random_comic(
    pool: web::Data<DbPool>,
    item_id: web::Path<u16>,
    query: web::Query<RandomItemComicQuery>,
) -> Result<Json<Option<u16>>> {
    let item_id = item_id.into_inner();
    let (include_guest_comics, include_non_canon_comics) =
        exclusion_to_filter_options(query.exclude);

    let comic_id = DatabaseComic::random_comic_id_with_item_id(
        &***pool,
        item_id,
        query.current_comic,
        include_guest_comics,
        include_non_canon_comics,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(Json(comic_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ComicId;

    #[test]
    fn no_exclusion_returns_all_nones() {
        assert_eq!(exclusion_to_filter_options(None), (None, None));
    }

    #[test]
    fn guest_exclusion_excludes_guest_comics() {
        assert_eq!(
            exclusion_to_filter_options(Some(Exclusion::Guest)),
            (Some(false), None)
        );
    }

    #[test]
    fn non_canon_exclusion_excludes_non_canon_comics() {
        assert_eq!(
            exclusion_to_filter_options(Some(Exclusion::NonCanon)),
            (None, Some(false))
        );
    }

    fn sample_row() -> ItemWithStats {
        ItemWithStats {
            id: 42,
            short_name: "sample".to_string(),
            name: "Sample Item".to_string(),
            r#type: "cast".to_string(),
            color_blue: 1,
            color_green: 2,
            color_red: 3,
            primary_image: None,
            start_comic_id: None,
            end_comic_id: None,
            first: Some(10),
            last: Some(20),
            count: 5,
            total_comics: 100,
            image_count: 1,
            first_image_id: Some(7),
        }
    }

    #[test]
    #[expect(
        clippy::float_cmp,
        reason = "exact value expected from an exact, non-computed input"
    )]
    fn build_item_response_maps_stats_fields() {
        let item = build_item_response(ItemId::from(42), sample_row()).unwrap();

        assert_eq!(item.first, ComicId::from_trusted(10));
        assert_eq!(item.last, ComicId::from_trusted(20));
        assert_eq!(item.appearances, 5);
        assert_eq!(item.total_comics, 100);
        assert_eq!(item.presence, 5.0);
        assert!(item.has_image);
    }

    #[test]
    fn build_item_response_uses_explicit_primary_image_when_set() {
        let row = ItemWithStats {
            primary_image: Some(99),
            ..sample_row()
        };

        let item = build_item_response(ItemId::from(42), row).unwrap();

        assert_eq!(item.primary_image, Some(99));
    }

    #[test]
    fn build_item_response_falls_back_to_first_image_id_when_primary_image_unset() {
        let row = ItemWithStats {
            primary_image: None,
            first_image_id: Some(7),
            ..sample_row()
        };

        let item = build_item_response(ItemId::from(42), row).unwrap();

        assert_eq!(item.primary_image, Some(7));
    }

    #[test]
    fn build_item_response_has_no_primary_image_when_none_available() {
        let row = ItemWithStats {
            primary_image: None,
            first_image_id: None,
            ..sample_row()
        };

        let item = build_item_response(ItemId::from(42), row).unwrap();

        assert_eq!(item.primary_image, None);
    }

    #[test]
    #[expect(
        clippy::float_cmp,
        reason = "exact value expected from an exact, non-computed input"
    )]
    fn build_item_response_zero_presence_when_no_comics_exist() {
        let row = ItemWithStats {
            total_comics: 0,
            ..sample_row()
        };

        let item = build_item_response(ItemId::from(42), row).unwrap();

        assert_eq!(item.presence, 0.0);
    }
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct RandomItemComicQuery {
    #[ts(optional)]
    exclude: Option<Exclusion>,
    #[serde(rename = "current-comic")]
    #[ts(type = "string")]
    pub current_comic: u16,
}
