use std::collections::HashSet;

use crate::models::v1::Exclusion;
use crate::models::v2::{ComicList, ItemId};
use actix_web::{error, web, HttpResponse, Result};
use actix_web_lab::extract::Query;
use database::models::{Comic as DatabaseComic, Occurrence as DatabaseOccurrence};
use database::DbPool;
use serde::Deserialize;
use tracing::info;

#[tracing::instrument(skip(pool))]
pub(crate) async fn all(
    pool: web::Data<DbPool>,
    query: web::Query<AllQuery>,
) -> Result<HttpResponse> {
    let (is_guest_comic, is_non_canon) = match query.exclude {
        None => (None, None),
        Some(Exclusion::Guest) => (Some(false), None),
        Some(Exclusion::NonCanon) => (None, Some(false)),
    };

    info!(
        "Requesting all comics (exclude guest comics: {}, exclude non-canon comics: {})",
        is_guest_comic.map_or(false, |v| !v),
        is_non_canon.map_or(false, |v| !v)
    );

    Ok(HttpResponse::Ok().json(fetch_comic_list(&pool, is_guest_comic, is_non_canon).await?))
}

#[tracing::instrument(skip(pool))]
pub(crate) async fn excluded(
    pool: web::Data<DbPool>,
    query: web::Query<ExcludedQuery>,
) -> Result<HttpResponse> {
    let (is_guest_comic, is_non_canon) = match query.exclusion {
        None => {
            return Err(error::ErrorBadRequest(
                "exclusion parameter must be set to either `guest` or `non-canon`",
            ))
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

    Ok(HttpResponse::Ok().json(fetch_comic_list(&pool, is_guest_comic, is_non_canon).await?))
}

#[tracing::instrument(skip(pool))]
pub(crate) async fn containing_items(
    pool: web::Data<DbPool>,
    query: Query<FilteredQuery>,
) -> Result<HttpResponse> {
    if query.item_ids.is_empty() {
        return Ok(HttpResponse::Ok().json([(); 0]));
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

    Ok(HttpResponse::Ok().json(appearances.remove(0)))
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

#[derive(Debug, Deserialize)]
pub(crate) struct AllQuery {
    exclude: Option<Exclusion>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ExcludedQuery {
    exclusion: Option<Exclusion>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FilteredQuery {
    #[serde(default, rename = "item-id")]
    item_ids: Vec<ItemId>,
}
