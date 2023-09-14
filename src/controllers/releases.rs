use std::sync::Arc;

use actix_web::{error, web, HttpRequest, HttpResponse, Result};
use arc_swap::ArcSwap;
use chrono::{DateTime, Duration, Utc};
use once_cell::sync::Lazy;
use rand::Rng;
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, info_span, trace, Instrument};

use crate::util::Either;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/{file}")
            .route(web::get().to(get_latest_script_file))
            .route(web::head().to(get_latest_script_file)),
    );
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ScriptFile {
    User,
    Meta,
}

impl ScriptFile {
    fn from_request(request: &str) -> Option<Self> {
        Some(match request {
            "qc-ext.latest.user.js" => Self::User,
            "qc-ext.latest.meta.js" => Self::Meta,
            _ => return None,
        })
    }

    fn github_filename(self) -> &'static str {
        match self {
            Self::User => "qc-ext.user.js",
            Self::Meta => "qc-ext.meta.js",
        }
    }
}

#[tracing::instrument(skip(req))]
pub(crate) async fn get_latest_script_file(
    file: web::Path<String>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let cache_result = validate_cache(&file, &req)?;
    let (script_file, cache, cache_expired) = match cache_result {
        Either::Left(cache_result) => cache_result,
        Either::Right(cache_response) => return Ok(cache_response),
    };

    debug!("Cache miss. Fetching asset from GitHub.");

    let (requested_asset, requested_asset_file) =
        fetch_asset_and_update_cache(&file, script_file, cache_expired, cache).await?;

    Ok(HttpResponse::Ok()
        .content_type("text/javascript; charset=utf-8")
        //.insert_header(("Content-Length", requested_asset.size))
        .insert_header(("ETag", requested_asset.node_id.clone()))
        .insert_header((
            "Last-Modified",
            requested_asset
                .updated_at
                .to_rfc2822()
                .replace("+0000", "GMT"),
        ))
        .body(requested_asset_file))
}

#[tracing::instrument(skip(cache))]
async fn fetch_asset_and_update_cache(
    file: &str,
    script_file: ScriptFile,
    cache_expired: bool,
    cache: arc_swap::Guard<Arc<ScriptCache>>,
) -> Result<(Asset, String), actix_web::Error> {
    let releases_response = Client::new()
        .get("https://api.github.com/repos/Questionable-Content-Extensions/client/releases/latest")
        .header(
            "User-Agent",
            "https://github.com/Questionable-Content-Extensions/server",
        )
        .send()
        .instrument(info_span!("fetch_latest_release"))
        .await
        .map_err(error::ErrorInternalServerError)?;
    let releases: Releases = releases_response
        .json()
        .instrument(info_span!("fetch_latest_release_json"))
        .await
        .map_err(error::ErrorInternalServerError)?;
    let requested_asset = releases
        .assets
        .into_iter()
        .find(|a| a.name == script_file.github_filename())
        .ok_or_else(|| {
            error::ErrorNotFound(format!(
                "According to GitHub, there is no latest release of {file}! \
                Hopefully this is a transient error, try again in a few minutes.",
            ))
        })?;
    debug!(?requested_asset);
    let asset_response = Client::new()
        .get(&requested_asset.browser_download_url)
        .header(
            "User-Agent",
            "https://github.com/Questionable-Content-Extensions/server",
        )
        .send()
        .instrument(info_span!("fetch_file", ?requested_asset.browser_download_url))
        .await
        .map_err(error::ErrorInternalServerError)?;
    let requested_asset_file = asset_response
        .text()
        .instrument(info_span!("fetch_file_text"))
        .await
        .map_err(error::ErrorInternalServerError)?;
    let new_cache = if cache_expired {
        // Start a new cache
        let (user, meta) = match script_file {
            ScriptFile::User => (
                Some((requested_asset.clone(), requested_asset_file.clone())),
                None,
            ),
            ScriptFile::Meta => (
                None,
                Some((requested_asset.clone(), requested_asset_file.clone())),
            ),
        };

        ScriptCache {
            expiration: Utc::now() + Duration::hours(1),
            user,
            meta,
        }
    } else {
        // Cache wasn't expired, but we still had to fetch the asset
        // which means it was a cache miss, so add the missing asset
        // to the existing cache
        let mut new_cache = (**cache).clone();
        if script_file == ScriptFile::User {
            new_cache.user = Some((requested_asset.clone(), requested_asset_file.clone()));
        } else {
            new_cache.meta = Some((requested_asset.clone(), requested_asset_file.clone()));
        }
        new_cache
    };
    trace!(?new_cache);
    SCRIPT_CACHE.compare_and_swap(cache, Arc::new(new_cache));
    Ok((requested_asset, requested_asset_file))
}

type ValidateCacheResult =
    Either<(ScriptFile, arc_swap::Guard<Arc<ScriptCache>>, bool), HttpResponse>;

#[tracing::instrument(skip(req))]
fn validate_cache(
    file: &web::Path<String>,
    req: &HttpRequest,
) -> Result<ValidateCacheResult, actix_web::Error> {
    let script_file = ScriptFile::from_request(file).ok_or_else(|| {
        error::ErrorBadRequest(format!("{file} is not a valid release file name"))
    })?;

    let jitter = rand::thread_rng().gen_range(-20..20);
    trace!(jitter);

    let cache = SCRIPT_CACHE.load();
    let cache_expired = cache.expiration - Utc::now() <= Duration::seconds(jitter);
    trace!(?cache.expiration, cache_expired);
    if !cache_expired {
        if let Some((cached_asset, cached_file)) = match script_file {
            ScriptFile::User => cache.user.as_ref(),
            ScriptFile::Meta => cache.meta.as_ref(),
        } {
            trace!(?cached_asset, cached_file);
            let etag = req
                .headers()
                .get("If-None-Match")
                .and_then(|h| h.to_str().ok());
            let last_modified = req
                .headers()
                .get("If-Modified-Since")
                .and_then(|h| h.to_str().ok());

            if let Some(etag) = etag {
                if etag == cached_asset.node_id {
                    debug!(etag, "ETag match");
                    return Ok(Either::Right(
                        HttpResponse::NotModified()
                            .insert_header(("ETag", cached_asset.node_id.clone()))
                            .finish(),
                    ));
                }
            }
            if let Some(last_modified) = last_modified {
                if last_modified == cached_asset.updated_at.to_rfc2822().replace("+0000", "GMT") {
                    debug!(last_modified, "Last Modified match");
                    return Ok(Either::Right(
                        HttpResponse::NotModified()
                            .insert_header(("ETag", cached_asset.node_id.clone()))
                            .finish(),
                    ));
                }
            }

            debug!("Cache miss in client; cache hit in server. Returning cached asset.");
            return Ok(Either::Right(
                HttpResponse::Ok()
                    .content_type("text/javascript; charset=utf-8")
                    .insert_header(("ETag", cached_asset.node_id.clone()))
                    .insert_header((
                        "Last-Modified",
                        cached_asset.updated_at.to_rfc2822().replace("+0000", "GMT"),
                    ))
                    .insert_header(("X-Script-Cached", "true"))
                    .body(cached_file.to_owned()),
            ));
        }
    }
    Ok(Either::Left((script_file, cache, cache_expired)))
}

#[derive(Debug, Deserialize)]
struct Releases {
    assets: Vec<Asset>,
}

#[derive(Clone, Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    node_id: String,
    updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
struct ScriptCache {
    expiration: DateTime<Utc>,
    user: Option<(Asset, String)>,
    meta: Option<(Asset, String)>,
}

static SCRIPT_CACHE: Lazy<ArcSwap<ScriptCache>> = Lazy::new(|| {
    ArcSwap::from_pointee(ScriptCache {
        expiration: Utc::now() - Duration::hours(1),
        user: None,
        meta: None,
    })
});
