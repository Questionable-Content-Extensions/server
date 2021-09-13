//! Questionable Content Extensions server.

// <editor-fold desc="Coding conventions" defaultstate="collapsed">
// Coding conventions
//
// Deny (don't do this)
#![deny(anonymous_parameters)]
#![deny(elided_lifetimes_in_paths)]
#![deny(ellipsis_inclusive_range_patterns)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![deny(trivial_numeric_casts)]
#![deny(unsafe_code)]
#![deny(rustdoc::broken_intra_doc_links)]
//#![deny(unused)]
//
// Warn (try not to do this)
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(variant_size_differences)]
//
// Clippy conventions
//
// Deny (don't do this)
#![deny(clippy::cast_lossless)]
#![deny(clippy::default_trait_access)]
#![deny(clippy::empty_enum)]
#![deny(clippy::enum_glob_use)]
#![deny(clippy::expl_impl_clone_on_copy)]
#![deny(clippy::explicit_into_iter_loop)]
#![deny(clippy::explicit_iter_loop)]
#![deny(clippy::manual_filter_map)]
#![deny(clippy::filter_map_next)]
#![deny(clippy::if_not_else)]
#![deny(clippy::invalid_upcast_comparisons)]
#![deny(clippy::items_after_statements)]
#![deny(clippy::large_digit_groups)]
#![deny(clippy::map_flatten)]
#![deny(clippy::match_same_arms)]
#![deny(clippy::mut_mut)]
#![deny(clippy::needless_continue)]
#![deny(clippy::needless_pass_by_value)]
#![deny(clippy::map_unwrap_or)]
#![deny(clippy::redundant_closure_for_method_calls)]
#![deny(clippy::single_match_else)]
#![deny(clippy::string_add_assign)]
#![deny(clippy::type_repetition_in_bounds)]
#![deny(clippy::unseparated_literal_suffix)]
#![deny(clippy::unused_self)]
#![deny(clippy::use_self)] // Sometimes gives false positives; feel free to disable.
#![deny(clippy::used_underscore_binding)]
//
// Warn (try not to do this)
#![warn(clippy::must_use_candidate)]
#![warn(clippy::enum_variant_names)]
#![warn(clippy::shadow_unrelated)]
#![warn(clippy::similar_names)]
#![warn(clippy::too_many_lines)]
// </editor-fold>

use crate::database::models::Token;
use crate::database::DbPool;
use crate::models::token_permissions;
use crate::util::{ComicUpdater, NewsUpdater};
use actix_web::dev::ServiceRequest;
use actix_web::{error, web, App, Error, FromRequest, HttpServer};
use actix_web_grants::GrantsMiddleware;
use anyhow::{anyhow, Context as _, Result};
use log::{error, info};
use serde::Deserialize;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use util::Environment;

mod controllers;
mod database;
mod models;

mod util;

#[actix_web::main]
async fn main() -> Result<()> {
    Environment::init();

    // Initialize logging
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "actix_web=info,qcext_server=info");
    }
    pretty_env_logger::init();

    let bind_address = format!("localhost:{}", Environment::port());
    info!("Starting server at: {}", &bind_address);

    let http_db_pool = DbPool::create().await;
    let db_pool = http_db_pool.clone();

    let http_news_updater: web::Data<NewsUpdater> = web::Data::new(NewsUpdater::new());
    let news_updater = Arc::clone(&http_news_updater);

    // Start HTTP server
    let start_http_server = move || -> Result<_> {
        Ok(HttpServer::new(move || {
            let auth = GrantsMiddleware::with_extractor(extract_permissions);
            App::new()
                .app_data(web::Data::new(http_db_pool.clone()))
                .app_data(http_news_updater.clone())
                .wrap(auth)
                .wrap(actix_web::middleware::Compress::default())
                .wrap(actix_web::middleware::Logger::default())
                .service(web::scope("/api").configure(controllers::api::configure))
        })
        .bind(&bind_address)?
        .run())
    };

    if Environment::background_services_enabled() {
        let background_news_updater_db_pool = db_pool.clone();
        let background_comic_updater_db_pool = db_pool;

        let background_news_updater = Arc::clone(&news_updater);
        let background_comic_news_updater = news_updater;

        let background_news_updater = tokio::task::spawn(async move {
            info!("Background news updater starting...");

            loop {
                match background_news_updater
                    .background_news_updater(&background_news_updater_db_pool)
                    .await
                {
                    Err(e) => {
                        error!("The background news updater returned an error: {}", e);
                        info!("Waiting one minute before starting up again.");
                        sleep(Duration::from_secs(60)).await;
                    }
                    _ => unreachable!(),
                }
            }
        });

        let background_comic_updater = tokio::task::spawn(async move {
            info!("Background comic updater starting...");

            let comic_updater = ComicUpdater::new();
            loop {
                match comic_updater
                    .background_comic_updater(
                        &background_comic_updater_db_pool,
                        &background_comic_news_updater,
                    )
                    .await
                {
                    Err(e) => {
                        error!("The background comic updater returned an error: {}", e);
                        info!("Waiting one minute before starting up again.");
                        sleep(Duration::from_secs(60)).await;
                    }
                    _ => unreachable!(),
                }
            }
        });

        let http_server = start_http_server()?;

        let (http_server_result, background_news_updating_result, background_comic_updating_result) = futures::join!(
            http_server,
            background_news_updater,
            background_comic_updater
        );
        http_server_result.context("actix_web::HttpServer crashed")?;
        background_news_updating_result.context("Background news updater crashed")?;
        background_comic_updating_result.context("Background comic updater crashed")?;
    } else {
        start_http_server()?
            .await
            .context("actix_web::HttpServer crashed")?;
    }

    Ok(())
}

#[allow(unsafe_code)]
#[warn(clippy::cast_ref_to_mut)]
async fn extract_permissions(request: &ServiceRequest) -> Result<Vec<String>, Error> {
    #[derive(Debug, Deserialize)]
    struct TokenQuery {
        token: Option<uuid::Uuid>,
    }

    let token = {
        // SAFETY: Yeah, no, this is completely bonkers, but I need the inner HttpRequest, so...
        let mut_request: &mut ServiceRequest = unsafe { &mut *(request as *const _ as *mut _) };

        let (request, payload) = mut_request.parts_mut();

        let token_query = web::Query::<TokenQuery>::from_request(&*request, &mut *payload).await?;
        if let Some(token) = token_query.token {
            token
        } else {
            let bytes = web::Bytes::from_request(&*request, &mut *payload).await?;
            let token_query: Result<TokenQuery, _> = serde_json::from_slice(&bytes[..]);
            if let Ok(token_query) = token_query {
                // Now that we've grabbed the token from the JSON payload, restore the payload.
                let mut payload = actix_http::h1::Payload::empty();
                payload.unread_data(bytes);
                mut_request.set_payload(payload.into());

                if let Some(token) = token_query.token {
                    token
                } else {
                    // If there is no token provided, there are no permissions
                    return Ok(vec![]);
                }
            } else {
                // If there is no token provided, there are no permissions
                return Ok(vec![]);
            }
        }
    };

    let pool = request
        .app_data::<web::Data<DbPool>>()
        .ok_or_else(|| anyhow!("Could not get DbPool from request"))
        .map_err(error::ErrorInternalServerError)?;
    let mut conn = pool
        .acquire()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let result = sqlx::query_as!(
        Token,
        r#"
            SELECT * FROM `token`
            WHERE `id` = ?
        "#,
        token.to_string()
    )
    .fetch_optional(&mut conn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    let token = if let Some(token) = result {
        token
    } else {
        // Invalid token provided, there are no permissions
        return Ok(vec![]);
    };

    let mut permissions = Vec::with_capacity(7);
    permissions.push(token_permissions::HAS_VALID_TOKEN.to_string());
    if token.CanAddItemToComic != 0 {
        permissions.push(token_permissions::CAN_ADD_ITEM_TO_COMIC.to_string());
    }
    if token.CanRemoveItemFromComic != 0 {
        permissions.push(token_permissions::CAN_REMOVE_ITEM_FROM_COMIC.to_string());
    }
    if token.CanChangeComicData != 0 {
        permissions.push(token_permissions::CAN_CHANGE_COMIC_DATA.to_string());
    }
    if token.CanAddImageToItem != 0 {
        permissions.push(token_permissions::CAN_ADD_IMAGE_TO_ITEM.to_string());
    }
    if token.CanRemoveImageFromItem != 0 {
        permissions.push(token_permissions::CAN_REMOVE_IMAGE_FROM_ITEM.to_string());
    }
    if token.CanChangeItemData != 0 {
        permissions.push(token_permissions::CAN_CHANGE_ITEM_DATA.to_string());
    }
    Ok(permissions)
}
