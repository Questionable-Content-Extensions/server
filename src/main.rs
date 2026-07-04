//! Questionable Content Extensions server.

use crate::models::Token;
use crate::util::{ComicUpdater, ComicUpdaterTrigger, Either, NewsUpdater, TokenPermissionsCache};
use actix_files::{Files, NamedFile};
use actix_http::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::web::PayloadConfig;
use actix_web::{App, Error, FromRequest, HttpServer, error, web};
use actix_web_grants::GrantsMiddleware;
use anyhow::{Context as _, Result, anyhow};
use database::DbPool;
use database::models::Token as DatabaseToken;
use database::models::stats::TopRankedStintRow as DbTopRankedStintRow;
use futures::stream::{FuturesUnordered, StreamExt};
use futures::{FutureExt, pin_mut};
use opentelemetry::KeyValue;
use opentelemetry::sdk::Resource;
use opentelemetry::sdk::trace::Tracer;
use opentelemetry::trace::TraceError;
use opentelemetry_otlp::WithExportConfig;
use serde::Deserialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{Duration, sleep};
use tracing::{Level, Span, error, info};
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder, TracingLogger};
use tracing_subscriber::layer::SubscriberExt;
use util::environment;
use uuid::Uuid;

#[macro_use]
mod util;

mod api;
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

    let http_db_pool = DbPool::create(
        environment::database_url(),
        environment::try_database_max_connections_u32().unwrap_or(50),
        environment::try_database_min_connections_u32().unwrap_or(5),
    )
    .await;
    let db_pool = http_db_pool.clone();

    info!("Running any outstanding database migrations...");
    database::migrate(&db_pool).await?;

    let bind_address = format!("0.0.0.0:{}", environment::port_u16());
    info!("Starting server at: {}", &bind_address);

    let http_news_updater: web::Data<NewsUpdater> = web::Data::new(NewsUpdater::new());
    let news_updater = Arc::clone(&http_news_updater);

    let http_token_cache: web::Data<TokenPermissionsCache> =
        web::Data::new(TokenPermissionsCache::new());

    let http_comic_updater_trigger: web::Data<ComicUpdaterTrigger> =
        web::Data::new(ComicUpdaterTrigger::new());
    let comic_updater_trigger = Arc::clone(&http_comic_updater_trigger);

    // Start HTTP server
    let start_http_server = move || -> Result<actix_web::dev::Server> {
        Ok(HttpServer::new(move || {
            let auth = GrantsMiddleware::with_extractor(extract_permissions);
            let a = App::new()
                .app_data(web::Data::new(http_db_pool.clone()))
                .app_data(http_news_updater.clone())
                .app_data(http_token_cache.clone())
                .app_data(http_comic_updater_trigger.clone())
                .app_data(PayloadConfig::new(1_048_576))
                .wrap(auth)
                .wrap(actix_web::middleware::Compress::default()).wrap(actix_web::middleware::Logger::new(
                    r#"%{X-Forwarded-For}i (%{X-Real-IP}i) (QCExt version %{X-QCExt-Version}i) "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#,
                ))
                .wrap(TracingLogger::<DomainRootSpanBuilder>::new());

            // Because of legacy reasons, the old API needs to be directly at the root.
            // Any newer APIs should be mounted *inside* v1's `configure`
            a.service(web::scope("/api").configure(api::configure))
                .service(web::scope("/releases").configure(controllers::releases::configure))
                .service(
                    Files::new("/", "./build/")
                        .index_file("index.html")
                        .default_handler(web::to(|| async {
                            NamedFile::open_async("./build/index.html")
                                .await
                                .map_err(error::ErrorInternalServerError)
                        })),
                )
        })
        .disable_signals()
        .bind(&bind_address)?
        .run())
    };

    let (shutdown_sender, mut background_news_updater_shutdown_receiver) = broadcast::channel(1);
    let mut shutdown_futures = FuturesUnordered::new();
    if environment::background_services_bool() {
        let background_news_updater_db_pool = db_pool.clone();
        let background_rank_stints_pool = db_pool.clone();
        let background_comic_updater_db_pool = db_pool;

        let background_news_updater = Arc::clone(&news_updater);
        let background_comic_news_updater = news_updater;
        let background_comic_updater_trigger = comic_updater_trigger;

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
                sleep(Duration::from_mins(1)).await;
            }
        });

        let background_comic_updater = tokio::task::spawn(async move {
            info!("Background comic updater starting...");

            let comic_updater = ComicUpdater::new();
            while let Err(e) = comic_updater
                .background_comic_updater(
                    &background_comic_updater_db_pool,
                    &background_comic_news_updater,
                    &background_comic_updater_trigger,
                    &mut background_comic_updater_shutdown_receiver,
                )
                .await
            {
                error!("The background comic updater returned an error: {}", e);
                info!("Waiting one minute before starting up again.");
                sleep(Duration::from_mins(1)).await;
            }
        });

        let mut background_rank_stints_shutdown = shutdown_sender.subscribe();
        let background_rank_stints_refresher = tokio::task::spawn(async move {
            info!("Background rank stints refresher starting...");
            loop {
                tokio::select! {
                    () = sleep(Duration::from_secs(30)) => {}
                    _ = background_rank_stints_shutdown.recv() => {
                        info!("Background rank stints refresher shutting down.");
                        break;
                    }
                }
                let conn = background_rank_stints_pool.acquire().await;
                let mut conn = match conn {
                    Ok(c) => c,
                    Err(e) => {
                        error!("rank stints refresher: pool acquire failed: {e}");
                        continue;
                    }
                };
                let dirty = match DbTopRankedStintRow::needs_refresh(&mut *conn).await {
                    Ok(v) => v,
                    Err(e) => {
                        error!("rank stints refresher: needs_refresh check failed: {e}");
                        continue;
                    }
                };
                if !dirty {
                    continue;
                }
                match DbTopRankedStintRow::refresh_cache(&mut conn).await {
                    Ok(()) => info!("rank stints refresher: cache refreshed"),
                    Err(e) => error!("rank stints refresher: refresh_cache failed: {e}"),
                }
            }
        });

        shutdown_futures.push(Either::Right(background_news_updater));
        shutdown_futures.push(Either::Right(background_comic_updater));
        shutdown_futures.push(Either::Right(background_rank_stints_refresher));
    } else {
        // Background services are off (dev mode): do a one-time startup refresh so the
        // stints cache is current without needing the background task running.
        let mut conn = db_pool.acquire().await?;
        match DbTopRankedStintRow::needs_refresh(&mut *conn).await {
            Ok(true) => {
                info!("Refreshing rank stints cache on startup (dev mode)...");
                match DbTopRankedStintRow::refresh_cache(&mut conn).await {
                    Ok(()) => info!("Startup rank stints refresh complete."),
                    Err(e) => error!("Startup rank stints refresh failed: {e}"),
                }
            }
            Ok(false) => {}
            Err(e) => error!("Startup rank stints refresh: needs_refresh check failed: {e}"),
        }
    }

    let http_server = start_http_server()?;
    let http_server_handle = http_server.handle();
    tokio::spawn(http_server);

    #[cfg(unix)]
    let mut sig_term_signal_stream = {
        use tokio::signal::unix::{SignalKind, signal};

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
            result?;
        }
    }

    Ok(())
}

// Older extension clients only know how to send the token as a `token` field in the
// JSON request body, so that path can't be removed outright. Query-string and
// `Authorization: Bearer` delivery are checked first since they don't require
// buffering the body, but the body-JSON fallback stays until those clients age out.
fn bearer_token_from_headers(http_request: &actix_web::HttpRequest) -> Option<Token> {
    let value = http_request
        .headers()
        .get(actix_web::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    let uuid_str = value.strip_prefix("Bearer ")?;
    Uuid::parse_str(uuid_str).ok().map(Token::from)
}

async fn extract_permissions(request: &mut ServiceRequest) -> Result<HashSet<String>, Error> {
    #[derive(Debug, Deserialize)]
    struct TokenQuery {
        token: Option<Token>,
    }

    let token_from_query_or_header = {
        let (http_request, payload) = request.parts_mut();

        let token_query =
            web::Query::<TokenQuery>::from_request(&*http_request, &mut *payload).await?;
        token_query
            .token
            .or_else(|| bearer_token_from_headers(http_request))
    };

    let token = if let Some(token) = token_from_query_or_header {
        token
    } else {
        let (http_request, payload) = request.parts_mut();

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
                return Ok(HashSet::new());
            }
        } else {
            // If there is no token provided, there are no permissions
            return Ok(HashSet::new());
        }
    };

    let token_str = token.to_string();

    let cache = request
        .app_data::<web::Data<TokenPermissionsCache>>()
        .ok_or_else(|| anyhow!("Could not get TokenPermissionsCache from request"))
        .map_err(error::ErrorInternalServerError)?;

    if let Some(permissions) = cache.get(&token_str) {
        return Ok(permissions);
    }

    let pool = request
        .app_data::<web::Data<DbPool>>()
        .ok_or_else(|| anyhow!("Could not get DbPool from request"))
        .map_err(error::ErrorInternalServerError)?;
    let mut conn = pool
        .acquire()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let permissions = DatabaseToken::get_permissions_for_token(&mut *conn, token_str.clone())
        .await
        .map_err(error::ErrorInternalServerError)?;

    cache.set(token_str, permissions.clone());

    Ok(permissions)
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
                .with_http_client(reqwest_011::Client::default())
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
        let headers: Vec<_> = request.headers().iter().collect();
        tracing_actix_web::root_span!(
            request,
            qcext.version = qcext_version,
            http.headers = ?headers
        )
    }

    fn on_request_end<B: MessageBody>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;

    #[test]
    fn bearer_token_from_headers_parses_valid_bearer_token() {
        let uuid = Uuid::new_v4();
        let request = TestRequest::default()
            .insert_header(("Authorization", format!("Bearer {uuid}")))
            .to_http_request();

        assert_eq!(bearer_token_from_headers(&request), Some(Token::from(uuid)));
    }

    #[test]
    fn bearer_token_from_headers_returns_none_without_header() {
        let request = TestRequest::default().to_http_request();

        assert_eq!(bearer_token_from_headers(&request), None);
    }

    #[test]
    fn bearer_token_from_headers_returns_none_for_non_bearer_scheme() {
        let uuid = Uuid::new_v4();
        let request = TestRequest::default()
            .insert_header(("Authorization", format!("Basic {uuid}")))
            .to_http_request();

        assert_eq!(bearer_token_from_headers(&request), None);
    }

    #[test]
    fn bearer_token_from_headers_returns_none_for_malformed_uuid() {
        let request = TestRequest::default()
            .insert_header(("Authorization", "Bearer not-a-uuid"))
            .to_http_request();

        assert_eq!(bearer_token_from_headers(&request), None);
    }
}
