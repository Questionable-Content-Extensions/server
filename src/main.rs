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
//#![warn(clippy::shadow_unrelated)]
#![warn(clippy::similar_names)]
#![warn(clippy::too_many_lines)]
// </editor-fold>

use crate::models::v1::Token;
use crate::util::{ComicUpdater, Either, NewsUpdater};
use actix_files::Files;
use actix_http::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::web::PayloadConfig;
use actix_web::{error, web, App, Error, FromRequest, HttpServer};
use actix_web_grants::GrantsMiddleware;
use anyhow::{anyhow, Context as _, Result};
use database::models::Token as DatabaseToken;
use database::DbPool;
use futures::stream::{FuturesUnordered, StreamExt};
use futures::{pin_mut, FutureExt};
use opentelemetry::sdk::trace::Tracer;
use opentelemetry::sdk::Resource;
use opentelemetry::trace::TraceError;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use tracing::{error, info, Level, Span};
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder, TracingLogger};
use tracing_subscriber::layer::SubscriberExt;
use util::environment;

#[macro_use]
mod util;

mod controllers;
mod models;

#[actix_web::main]
async fn main() -> Result<()> {
    environment::init_dotenv();

    let filter = tracing_subscriber::filter::Targets::new()
        .with_target("hyper", Level::INFO)
        .with_target("actix_http", Level::DEBUG)
        .with_target("actix_server", Level::DEBUG)
        .with_default(Level::TRACE);

    let tracer = init_tracer().unwrap();
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = tracing_subscriber::Registry::default()
        .with(filter)
        .with(tracing_subscriber::fmt::Layer::default())
        .with(telemetry);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let http_db_pool = DbPool::create(environment::database_url()).await;
    let db_pool = http_db_pool.clone();

    info!("Running any outstanding database migrations...");
    database::migrate(&db_pool).await?;

    let bind_address = format!("0.0.0.0:{}", environment::port_u16());
    info!("Starting server at: {}", &bind_address);

    let http_news_updater: web::Data<NewsUpdater> = web::Data::new(NewsUpdater::new());
    let news_updater = Arc::clone(&http_news_updater);

    // Start HTTP server
    let start_http_server = move || -> Result<actix_web::dev::Server> {
        Ok(HttpServer::new(move || {
            let auth = GrantsMiddleware::with_extractor(extract_permissions);
            let a = App::new()
                .app_data(web::Data::new(http_db_pool.clone()))
                .app_data(http_news_updater.clone())
                .app_data(PayloadConfig::new(1048576))
                .wrap(auth)
                .wrap(actix_web::middleware::Compress::default()).wrap(actix_web::middleware::Logger::new(
                    r#"%{X-Forwarded-For}i (%{X-Real-IP}i) (QCExt version %{X-QCExt-Version}i) "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#,
                ))
                .wrap(TracingLogger::<DomainRootSpanBuilder>::new());

            // Because of legacy reasons, the old API needs to be directly at the root.
            // Any newer APIs should be mounted *inside* v1's `configure`
            a.service(web::scope("/api").configure(controllers::v1::api::configure))
                .service(web::scope("/releases").configure(controllers::releases::configure))
                .service(Files::new("/", "./build/").index_file("index.html"))
        })
        .disable_signals()
        .bind(&bind_address)?
        .run())
    };

    let (shutdown_sender, mut background_news_updater_shutdown_receiver) = broadcast::channel(1);
    let mut shutdown_futures = FuturesUnordered::new();
    if environment::background_services_bool() {
        let background_news_updater_db_pool = db_pool.clone();
        let background_comic_updater_db_pool = db_pool;

        let background_news_updater = Arc::clone(&news_updater);
        let background_comic_news_updater = news_updater;

        let mut background_comic_updater_shutdown_receiver = shutdown_sender.subscribe();

        let background_news_updater = tokio::task::spawn(async move {
            info!("Background news updater starting...");

            while let Err(e) = background_news_updater
                .background_news_updater(
                    &background_news_updater_db_pool,
                    &mut background_news_updater_shutdown_receiver,
                )
                .await
            {
                error!("The background news updater returned an error: {}", e);
                info!("Waiting one minute before starting up again.");
                sleep(Duration::from_secs(60)).await;
            }
        });

        let background_comic_updater = tokio::task::spawn(async move {
            info!("Background comic updater starting...");

            let comic_updater = ComicUpdater::new();
            while let Err(e) = comic_updater
                .background_comic_updater(
                    &background_comic_updater_db_pool,
                    &background_comic_news_updater,
                    &mut background_comic_updater_shutdown_receiver,
                )
                .await
            {
                error!("The background comic updater returned an error: {}", e);
                info!("Waiting one minute before starting up again.");
                sleep(Duration::from_secs(60)).await;
            }
        });

        shutdown_futures.push(Either::Right(background_news_updater));
        shutdown_futures.push(Either::Right(background_comic_updater));
    }

    let http_server = start_http_server()?;
    let http_server_handle = http_server.handle();
    tokio::spawn(http_server);

    #[cfg(unix)]
    let mut sig_term_signal_stream = {
        use tokio::signal::unix::{signal, SignalKind};

        signal(SignalKind::terminate())?
    };
    let sig_term = {
        #[cfg(unix)]
        {
            sig_term_signal_stream.recv().fuse()
        }
        #[cfg(not(unix))]
        {
            use futures::future::pending;

            // On non-Unix systems, just use a dummy never-ready future
            // to make the compiler happy below.
            pending::<Option<()>>()
        }
    };
    pin_mut!(sig_term);

    let ctrl_c = tokio::signal::ctrl_c().fuse();
    pin_mut!(ctrl_c);
    if shutdown_futures.is_empty() {
        futures::select! {
            sigint = ctrl_c => sigint.context("tokio::signal::ctrl_c failed")?,
            sigterm = sig_term => sigterm.context("tokio::signal::terminate failed")?,
        }
    } else {
        #[allow(clippy::mut_mut)]
        {
            futures::select! {
                sigint = ctrl_c => sigint.context("tokio::signal::ctrl_c failed")?,
                sigterm = sig_term => sigterm.context("tokio::signal::terminate failed")?,
                either = shutdown_futures.next().fuse() => match either {
                    Some(Either::Right(result)) => result.context("Background service crashed")?,
                    _ => unreachable!()
                },
            };
        }
    }

    // Receivers should be live at this point, although either or both of the
    // services may have crashed, so let's not really assert anything about it.
    info!("Shutting down HTTP server!");
    let _ = shutdown_sender.send(());

    shutdown_futures.push(Either::Left(http_server_handle.stop(true)));

    while let Some(either) = shutdown_futures.next().await {
        if let Either::Right(result) = either {
            result?
        }
    }

    Ok(())
}

async fn extract_permissions(request: &mut ServiceRequest) -> Result<Vec<String>, Error> {
    #[derive(Debug, Deserialize)]
    struct TokenQuery {
        token: Option<Token>,
    }

    let token = {
        let (http_request, payload) = request.parts_mut();

        let token_query =
            web::Query::<TokenQuery>::from_request(&*http_request, &mut *payload).await?;
        if let Some(token) = token_query.token {
            token
        } else {
            // Grab the payload and try parsing it as JSON
            let bytes = web::Bytes::from_request(&*http_request, &mut *payload).await?;
            let token_query: Result<TokenQuery, _> = serde_json::from_slice(&bytes[..]);

            // Now that we've grabbed the payload, we need to restore the payload
            // for the rest of the Actix machinery to do its thing.
            let (_, mut payload) = actix_http::h1::Payload::create(true);
            payload.unread_data(bytes);
            request.set_payload(payload.into());

            if let Ok(token_query) = token_query {
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

    DatabaseToken::get_permissions_for_token(&mut conn, token.to_string())
        .await
        .map_err(error::ErrorInternalServerError)
}

fn init_tracer() -> Result<Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(
            opentelemetry::sdk::trace::config().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                "qcext-server",
            )])),
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint("https://api.honeycomb.io/v1/traces")
                .with_http_client(reqwest::Client::default())
                .with_headers(HashMap::from([
                    ("x-honeycomb-dataset".into(), "qcext-server-dataset".into()),
                    (
                        "x-honeycomb-team".into(),
                        environment::honeycomb_key().into(),
                    ),
                ]))
                .with_timeout(Duration::from_secs(2)),
        ) // Replace with runtime::Tokio if using async main
        .install_batch(opentelemetry::runtime::Tokio)
}

struct DomainRootSpanBuilder;

impl RootSpanBuilder for DomainRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        let qcext_version = request
            .headers()
            .get("X-QCExt-Version")
            .map(|h| String::from_utf8_lossy(h.as_ref()).into_owned());
        tracing_actix_web::root_span!(request, qcext.version = qcext_version)
    }

    fn on_request_end<B: MessageBody>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}
