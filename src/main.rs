//! Questionable Content Extensions server.

use crate::models::Token;
use crate::util::{ComicUpdater, Either, NewsUpdater};
use actix_files::{Files, NamedFile};
use actix_http::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::web::PayloadConfig;
use actix_web::{App, Error, FromRequest, HttpServer, error, web};
use actix_web_grants::GrantsMiddleware;
use anyhow::{Context as _, Result, anyhow};
use database::DbPool;
use database::models::Token as DatabaseToken;
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
                    &mut background_comic_updater_shutdown_receiver,
                )
                .await
            {
                error!("The background comic updater returned an error: {}", e);
                info!("Waiting one minute before starting up again.");
                sleep(Duration::from_mins(1)).await;
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

async fn extract_permissions(request: &mut ServiceRequest) -> Result<HashSet<String>, Error> {
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
                    return Ok(HashSet::new());
                }
            } else {
                // If there is no token provided, there are no permissions
                return Ok(HashSet::new());
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

    DatabaseToken::get_permissions_for_token(&mut *conn, token.to_string())
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
