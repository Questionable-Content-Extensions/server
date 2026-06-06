use crate::api::v3::models::{ComicList, Exclusion, Item, ItemColor, ItemType, RelatedItem};
use crate::models::ItemId;
use actix_web::{HttpResponse, Result, error, web};
use anyhow::anyhow;
use database::DbPool;
use database::models::{
    Comic as DatabaseComic, Item as DatabaseItem, RelatedItem as RelatedDatabaseItem,
};
use serde::Deserialize;
use std::convert::{TryFrom, TryInto};
use tracing::{Instrument, info_span};
use ts_rs::TS;

#[expect(
    clippy::cast_precision_loss,
    reason = "comic/item counts are well within f64 mantissa precision"
)]
#[tracing::instrument(skip(pool))]
pub async fn by_id(pool: web::Data<DbPool>, item_id: web::Path<ItemId>) -> Result<HttpResponse> {
    let item_id = item_id.into_inner();

    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let item = DatabaseItem::by_id(&mut *conn, item_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound(anyhow!("No item with id {item_id} exists")))?;

    let primary_image = if let Some(primary_image) = item.primary_image {
        Some(primary_image)
    } else {
        let image_metadatas = DatabaseItem::image_metadatas_by_id(&***pool, item_id.into_inner())
            .await
            .map_err(error::ErrorInternalServerError)?;

        image_metadatas.first().map(|first_image| first_image.id)
    };

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
        primary_image,
    };

    Ok(HttpResponse::Ok().json(item))
}

#[tracing::instrument(skip(pool))]
pub async fn friends(pool: web::Data<DbPool>, item_id: web::Path<u16>) -> Result<HttpResponse> {
    let items = related_items(pool, *item_id, ItemType::Cast, 5).await?;

    Ok(HttpResponse::Ok().json(items))
}

#[tracing::instrument(skip(pool))]
pub async fn locations(pool: web::Data<DbPool>, item_id: web::Path<u16>) -> Result<HttpResponse> {
    let items = related_items(pool, *item_id, ItemType::Location, 5).await?;

    Ok(HttpResponse::Ok().json(items))
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

#[tracing::instrument(skip(pool))]
pub async fn comics(pool: web::Data<DbPool>, item_id: web::Path<u16>) -> Result<HttpResponse> {
    let item_id = item_id.into_inner();

    let comics: Vec<ComicList> =
        DatabaseComic::all_with_item_id_mapped(&***pool, item_id, From::from)
            .await
            .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(comics))
}

const fn exclusion_to_filter_options(exclude: Option<Exclusion>) -> (Option<bool>, Option<bool>) {
    match exclude {
        None => (None, None),
        Some(Exclusion::Guest) => (Some(false), None),
        Some(Exclusion::NonCanon) => (None, Some(false)),
    }
}

#[tracing::instrument(skip(pool))]
pub async fn random_comic(
    pool: web::Data<DbPool>,
    item_id: web::Path<u16>,
    query: web::Query<RandomItemComicQuery>,
) -> Result<HttpResponse> {
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

    Ok(HttpResponse::Ok().json(comic_id))
}

#[cfg(test)]
mod tests {
    use super::*;

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
