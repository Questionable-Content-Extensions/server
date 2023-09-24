use crate::api::v2::models::{ComicList, Exclusion, Item, ItemColor, ItemType, RelatedItem};
use crate::models::ItemId;
use actix_web::{error, web, HttpResponse, Result};
use anyhow::anyhow;
use database::models::{
    Comic as DatabaseComic, Item as DatabaseItem, RelatedItem as RelatedDatabaseItem,
};
use database::DbPool;
use rand::seq::SliceRandom;
use serde::Deserialize;
use std::convert::{TryFrom, TryInto};
use tracing::{info_span, Instrument};

#[tracing::instrument(skip(pool))]
pub(crate) async fn by_id(
    pool: web::Data<DbPool>,
    item_id: web::Path<ItemId>,
) -> Result<HttpResponse> {
    let item_id = item_id.into_inner();

    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let item = DatabaseItem::by_id(&mut *conn, item_id.into_inner())
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
        DatabaseItem::first_and_last_apperance_and_count_by_id(&mut *conn, item_id.into_inner())
            .await
            .map_err(error::ErrorInternalServerError)?;

    let total_comics = DatabaseComic::count(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let image_count = DatabaseItem::image_count_by_id(&mut *conn, item_id.into_inner())
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

#[tracing::instrument(skip(pool))]
pub(crate) async fn friends(
    pool: web::Data<DbPool>,
    item_id: web::Path<u16>,
) -> Result<HttpResponse> {
    let items = related_items(pool, *item_id, ItemType::Cast, 5).await?;

    Ok(HttpResponse::Ok().json(items))
}

#[tracing::instrument(skip(pool))]
pub(crate) async fn locations(
    pool: web::Data<DbPool>,
    item_id: web::Path<u16>,
) -> Result<HttpResponse> {
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
pub(crate) async fn comics(
    pool: web::Data<DbPool>,
    item_id: web::Path<u16>,
) -> Result<HttpResponse> {
    let item_id = item_id.into_inner();

    let comics: Vec<ComicList> =
        DatabaseComic::all_with_item_id_mapped(&***pool, item_id, From::from)
            .await
            .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(comics))
}

#[tracing::instrument(skip(pool))]
pub(crate) async fn random_comic(
    pool: web::Data<DbPool>,
    item_id: web::Path<u16>,
    query: web::Query<RandomItemComicQuery>,
) -> Result<HttpResponse> {
    let item_id = item_id.into_inner();

    let (include_guest_comics, include_non_canon_comics) = match query.exclude {
        None => (None, None),
        Some(Exclusion::Guest) => (Some(false), None),
        Some(Exclusion::NonCanon) => (None, Some(false)),
    };

    let comics: Vec<ComicList> =
        DatabaseComic::all_with_item_id_mapped(&***pool, item_id, From::from)
            .await
            .map_err(error::ErrorInternalServerError)?;

    if comics.len() < 2 {
        return Ok(HttpResponse::Ok().json(()));
    }

    let mut thread_rng = rand::thread_rng();
    let comic = loop {
        let comic = comics.choose(&mut thread_rng).unwrap();
        if !include_guest_comics.unwrap_or(true) && comic.is_guest_comic {
            continue;
        }
        if !include_non_canon_comics.unwrap_or(true) && comic.is_non_canon {
            continue;
        }
        if comic.comic.into_inner() == query.current_comic {
            continue;
        }
        break comic;
    };

    Ok(HttpResponse::Ok().json(comic.comic))
}

#[derive(Debug, Deserialize)]
pub(crate) struct RandomItemComicQuery {
    exclude: Option<Exclusion>,
    #[serde(rename = "current-comic")]
    pub current_comic: u16,
}
