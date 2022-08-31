use std::sync::Arc;

use actix_web::{error, web, HttpResponse, Result};
use arc_swap::ArcSwap;
use chrono::{DateTime, Duration, Utc};
use once_cell::sync::Lazy;
use rand::Rng;
use reqwest::Client;
use serde::Deserialize;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/{file}").route(web::get().to(get_latest_script_file)));
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

pub(crate) async fn get_latest_script_file(file: web::Path<String>) -> Result<HttpResponse> {
    let script_file = ScriptFile::from_request(&file).ok_or_else(|| {
        error::ErrorBadRequest(format!("{file} is not a valid release file name"))
    })?;

    let jitter = rand::thread_rng().gen_range(-20..20);
    let cache = SCRIPT_CACHE.load();
    let cache_expired = cache.expiration - Utc::now() <= Duration::seconds(jitter);
    if !cache_expired {
        if let Some(cached_file) = match script_file {
            ScriptFile::User => cache.user.as_deref(),
            ScriptFile::Meta => cache.meta.as_deref(),
        } {
            return Ok(HttpResponse::Ok()
                .content_type("text/javascript; charset=utf-8")
                .body(cached_file.to_owned()));
        }
    }

    let releases_response = Client::new()
        .get("https://api.github.com/repos/Questionable-Content-Extensions/client/releases/latest")
        .header(
            "User-Agent",
            "https://github.com/Questionable-Content-Extensions/server",
        )
        .send()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let releases: Releases = releases_response
        .json()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let requested_asset_url = releases
        .assets
        .into_iter()
        .find(|a| a.name == script_file.github_filename())
        .ok_or_else(|| {
            error::ErrorNotFound(format!(
                "According to GitHub, there is no latest release of {}! \
                Hopefully this is a transient error, try again in a few minutes.",
                file
            ))
        })?
        .browser_download_url;

    let asset_response = Client::new()
        .get(requested_asset_url)
        .header(
            "User-Agent",
            "https://github.com/Questionable-Content-Extensions/server",
        )
        .send()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let asset = asset_response
        .text()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let new_cache = if cache_expired {
        // Start a new cache
        ScriptCache {
            expiration: Utc::now() + Duration::hours(1),
            user: if script_file == ScriptFile::User {
                Some(asset.clone())
            } else {
                None
            },
            meta: if script_file == ScriptFile::Meta {
                Some(asset.clone())
            } else {
                None
            },
        }
    } else {
        // Cache wasn't expired, but we still had to fetch the asset
        // which means it was a cache miss, so add the missing asset
        // to the existing cache
        let mut new_cache = (&**cache).clone();
        if script_file == ScriptFile::User {
            new_cache.user = Some(asset.clone());
        } else {
            new_cache.meta = Some(asset.clone());
        }
        new_cache
    };

    SCRIPT_CACHE.compare_and_swap(cache, Arc::new(new_cache));

    Ok(HttpResponse::Ok()
        .content_type("text/javascript; charset=utf-8")
        .body(asset))
}

#[derive(Debug, Deserialize)]
struct Releases {
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

#[derive(Clone, Debug)]
struct ScriptCache {
    expiration: DateTime<Utc>,
    user: Option<String>,
    meta: Option<String>,
}

static SCRIPT_CACHE: Lazy<ArcSwap<ScriptCache>> = Lazy::new(|| {
    ArcSwap::from_pointee(ScriptCache {
        expiration: Utc::now() - Duration::days(1),
        user: None,
        meta: None,
    })
});
