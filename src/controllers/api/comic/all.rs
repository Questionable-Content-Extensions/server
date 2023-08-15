use crate::models::{ComicList, Exclusion};
use actix_web::{error, web, HttpResponse, Result};
use database::models::Comic as DatabaseComic;
use database::DbPool;
use log::info;
use serde::Deserialize;

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
