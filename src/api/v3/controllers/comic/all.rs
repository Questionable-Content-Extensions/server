use actix_web::web::Json;
use actix_web::{Result, error, web};
use actix_web_lab::extract::Query;
use api_macros::api_endpoint;
use database::DbPool;
use database::models::{Comic as DatabaseComic, Occurrence as DatabaseOccurrence};
use serde::Deserialize;
use std::collections::HashSet;
use tracing::info;
use ts_rs::TS;

use crate::api::v3::models::{ComicList, Exclusion};
use crate::models::ItemId;

#[api_endpoint(method = "GET", path = "comicdata/")]
#[tracing::instrument(skip(pool))]
pub async fn all(
    pool: web::Data<DbPool>,
    query: web::Query<AllQuery>,
) -> Result<Json<Vec<ComicList>>> {
    let (is_guest_comic, is_non_canon) = match query.exclude {
        None => (None, None),
        Some(Exclusion::Guest) => (Some(false), None),
        Some(Exclusion::NonCanon) => (None, Some(false)),
    };

    info!(
        "Requesting all comics (exclude guest comics: {}, exclude non-canon comics: {})",
        is_guest_comic.is_some_and(|v| !v),
        is_non_canon.is_some_and(|v| !v)
    );

    Ok(Json(
        fetch_comic_list(&pool, is_guest_comic, is_non_canon).await?,
    ))
}

#[api_endpoint(method = "GET", path = "comicdata/excluded")]
#[tracing::instrument(skip(pool))]
pub async fn excluded(
    pool: web::Data<DbPool>,
    query: web::Query<ExcludedQuery>,
) -> Result<Json<Vec<ComicList>>> {
    let (is_guest_comic, is_non_canon) = match query.exclusion {
        None => {
            return Err(error::ErrorBadRequest(
                "exclusion parameter must be set to either `guest` or `non-canon`",
            ));
        }
        Some(Exclusion::Guest) => (Some(true), None),
        Some(Exclusion::NonCanon) => (None, Some(true)),
    };

    info!(
        "Requesting excluded {} comics",
        if is_guest_comic.is_some() {
            "guest"
        } else {
            "non-canon"
        }
    );

    Ok(Json(
        fetch_comic_list(&pool, is_guest_comic, is_non_canon).await?,
    ))
}

#[api_endpoint(method = "GET", path = "comicdata/containing-items")]
#[tracing::instrument(skip(pool))]
pub async fn containing_items(
    pool: web::Data<DbPool>,
    query: Query<FilteredQuery>,
) -> Result<Json<HashSet<u16>>> {
    if query.item_ids.is_empty() {
        return Ok(Json(HashSet::new()));
    }

    let mut appearances = Vec::new();
    for item_id in query.item_ids.iter().copied().map(ItemId::into_inner) {
        appearances.push(HashSet::<u16>::from_iter(
            DatabaseOccurrence::comic_id_occurrence_by_item_id(&***pool, item_id)
                .await
                .map_err(error::ErrorInternalServerError)?,
        ));
    }

    // Repeatedly whittle down the largest set until we have the intersection of all of them
    appearances.sort_by_key(HashSet::len);
    let (intersection, others) = appearances.split_at_mut(1);
    let intersection = &mut intersection[0];
    for other in others {
        intersection.retain(|e| other.contains(e));

        // If we end up with no overlap, we might as well bail out early.
        if intersection.is_empty() {
            break;
        }
    }

    Ok(Json(appearances.remove(0)))
}

#[tracing::instrument(skip(pool))]
async fn fetch_comic_list(
    pool: &DbPool,
    is_guest_comic: Option<bool>,
    is_non_canon: Option<bool>,
) -> Result<Vec<ComicList>> {
    let comics: Vec<ComicList> =
        DatabaseComic::all_with_mapping(&**pool, is_guest_comic, is_non_canon, From::from)
            .await
            .map_err(error::ErrorInternalServerError)?;

    Ok(comics)
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct AllQuery {
    #[ts(optional)]
    exclude: Option<Exclusion>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct ExcludedQuery {
    #[ts(optional)]
    exclusion: Option<Exclusion>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct FilteredQuery {
    #[serde(default, rename = "item-id")]
    #[ts(optional, rename = "item-id")]
    item_ids: Vec<ItemId>,
}
