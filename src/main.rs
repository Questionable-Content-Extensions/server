//! Questionable Content Extensions server.

use crate::models::Token;
use crate::util::{
    ComicUpdater, ComicUpdaterTrigger, Either, NewsUpdater, TokenPermissionsCache,
    hide_server_error_details,
};
use actix_files::{Files, NamedFile};
use actix_http::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::web::PayloadConfig;
use actix_web::{App, Error, FromRequest, HttpMessage, HttpServer, error, web};
use actix_web_grants::GrantsMiddleware;
use anyhow::{Context as _, Result, anyhow};
use database::DbPool;
use database::models::Token as DatabaseToken;
use database::models::stats::TopRankedStintRow as DbTopRankedStintRow;
use futures::stream::{FuturesUnordered, StreamExt};
use futures::{FutureExt, pin_mut};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::{ExporterBuildError, WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::{SdkTracerProvider, TracerProviderBuilder};
use serde::Deserialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{Duration, sleep};
use tracing::{Level, Span, error, info};
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder, TracingLogger};
use tracing_subscriber::filter::Targets;
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

    let filter = tracing_filter();

    let tracer_provider = init_tracer_provider(environment::honeycomb_key()).unwrap();
    let _tracer_provider_guard = TracerProviderGuard(tracer_provider.clone());
    let telemetry =
        tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer(SERVICE_NAME));
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
                .wrap(hide_server_error_details())
                .wrap(actix_web::middleware::Compress::default()).wrap(actix_web::middleware::Logger::new(
                    r#"%{X-Forwarded-For}i (%{X-Real-IP}i) (QCExt version %{X-QCExt-Version}i) "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#,
                ))
                .wrap(TracingLogger::<DomainRootSpanBuilder>::new());

            a.service(web::scope("/api").configure(api::configure))
                .service(web::scope("/releases").configure(controllers::releases::configure))
                .service(
                    Files::new("/", "./build/")
                        .index_file("index.html")
                        .default_handler(web::to(|| open_named_file("./build/index.html"))),
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

/// Opens `path` on the blocking thread pool, mapping any failure to a 500 response.
async fn open_named_file(path: &'static str) -> Result<NamedFile, Error> {
    web::block(move || NamedFile::open(path))
        .await
        .map_err(error::ErrorInternalServerError)?
        .map_err(error::ErrorInternalServerError)
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
    } else if !has_json_body(request.request()) {
        // Handlers of other body types, such as the v2 multipart image upload, look up their
        // own token, so their bodies are left unbuffered and subject only to their own limits.
        return Ok(HashSet::new());
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

    // Stash the resolved token in the request extensions so handlers that need it
    // (e.g. for audit-log attribution) can pull it via `web::ReqData<Token>`
    // instead of requiring a redundant `token` field in their request body.
    request.extensions_mut().insert(token);

    let token_str = token.to_string();

    let cache = request
        .app_data::<web::Data<TokenPermissionsCache>>()
        .ok_or_else(|| anyhow!("Could not get TokenPermissionsCache from request"))
        .map_err(error::ErrorInternalServerError)?;

    let pool = request
        .app_data::<web::Data<DbPool>>()
        .ok_or_else(|| anyhow!("Could not get DbPool from request"))
        .map_err(error::ErrorInternalServerError)?;

    permissions_for_method(cache, request.method(), token_str, |token_str| async move {
        let mut conn = pool
            .acquire()
            .await
            .map_err(error::ErrorInternalServerError)?;
        DatabaseToken::get_permissions_for_token(&mut *conn, token_str)
            .await
            .map_err(error::ErrorInternalServerError)
    })
    .await
}

/// Whether the request declares a JSON body, using the same content types `web::Json` accepts.
fn has_json_body(http_request: &actix_web::HttpRequest) -> bool {
    matches!(
        http_request.mime_type(),
        Ok(Some(mime)) if mime.subtype() == actix_web::mime::JSON
            || mime.suffix() == Some(actix_web::mime::JSON)
    )
}

/// Resolves `token`'s permissions, serving safe methods from `cache` and always calling `lookup`
/// for mutating ones, so a revoked permission stops authorizing changes immediately.
async fn permissions_for_method<F, Fut>(
    cache: &TokenPermissionsCache,
    method: &actix_web::http::Method,
    token: String,
    lookup: F,
) -> Result<HashSet<String>, Error>
where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = Result<HashSet<String>, Error>>,
{
    if method.is_safe()
        && let Some(permissions) = cache.get(&token)
    {
        return Ok(permissions);
    }

    let permissions = lookup(token.clone()).await?;
    cache.set(token, permissions.clone());

    Ok(permissions)
}

/// Builds the per-target level filter for the tracing subscriber.
///
/// `h2` is capped because the OTLP exporter talks HTTP/2 to Honeycomb: at `TRACE`, every export
/// emits `h2` spans that are themselves exported, sustaining a telemetry feedback loop.
fn tracing_filter() -> Targets {
    Targets::new()
        .with_target("h2", Level::INFO)
        .with_target("hyper", Level::INFO)
        .with_target("actix_http", Level::DEBUG)
        .with_target("actix_server", Level::DEBUG)
        .with_default(Level::TRACE)
}

const SERVICE_NAME: &str = "qcext-server";

/// Builds the tracer provider that batches spans and exports them to Honeycomb over OTLP/HTTP.
///
/// The exporter uses the blocking reqwest client because the batch span processor exports from
/// its own thread, which has no tokio runtime.
fn init_tracer_provider(honeycomb_key: &str) -> Result<SdkTracerProvider, ExporterBuildError> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint("https://api.honeycomb.io/v1/traces")
        .with_headers(HashMap::from([
            ("x-honeycomb-dataset".into(), "qcext-server-dataset".into()),
            ("x-honeycomb-team".into(), honeycomb_key.into()),
        ]))
        .with_timeout(Duration::from_secs(2))
        .build()?;

    Ok(tracer_provider_builder()
        .with_batch_exporter(exporter)
        .build())
}

fn tracer_provider_builder() -> TracerProviderBuilder {
    SdkTracerProvider::builder()
        .with_resource(Resource::builder().with_service_name(SERVICE_NAME).build())
}

/// Shuts down the tracer provider when dropped, so spans still queued for export are flushed on
/// every exit path out of `main`, including early error returns.
#[derive(Debug)]
struct TracerProviderGuard(SdkTracerProvider);

impl Drop for TracerProviderGuard {
    fn drop(&mut self) {
        if let Err(e) = self.0.shutdown() {
            error!("Failed to shut down the tracer provider: {e}");
        }
    }
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
    use actix_web::http::StatusCode;
    use actix_web::test::TestRequest;
    use actix_web::{Responder, body};

    #[actix_web::test]
    async fn open_named_file_serves_existing_file_contents() {
        let file = open_named_file("./Cargo.toml")
            .await
            .expect("Cargo.toml exists in the package root");
        let request = TestRequest::default().to_http_request();

        let response = file.respond_to(&request);
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(bytes, std::fs::read("./Cargo.toml").unwrap());
    }

    #[actix_web::test]
    async fn open_named_file_maps_missing_file_to_internal_server_error() {
        let error = open_named_file("./does-not-exist/index.html")
            .await
            .expect_err("the file does not exist");

        assert_eq!(
            error.as_response_error().status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn tracing_filter_caps_noisy_dependency_targets() {
        let filter = tracing_filter();

        assert!(filter.would_enable("h2", &Level::INFO));
        assert!(!filter.would_enable("h2", &Level::DEBUG));
        assert!(!filter.would_enable("h2::proto::connection", &Level::TRACE));
        assert!(filter.would_enable("hyper", &Level::INFO));
        assert!(!filter.would_enable("hyper", &Level::DEBUG));
        assert!(filter.would_enable("actix_http", &Level::DEBUG));
        assert!(!filter.would_enable("actix_http", &Level::TRACE));
        assert!(filter.would_enable("actix_server", &Level::DEBUG));
        assert!(!filter.would_enable("actix_server", &Level::TRACE));
    }

    #[test]
    fn tracing_filter_enables_trace_for_other_targets() {
        assert!(tracing_filter().would_enable("qcext_server", &Level::TRACE));
    }

    #[test]
    fn bearer_token_from_headers_parses_valid_bearer_token() {
        let uuid = Uuid::new_v4();
        let request = TestRequest::default()
            .insert_header(("Authorization", format!("Bearer {uuid}")))
            .to_http_request();

        assert_eq!(bearer_token_from_headers(&request), Some(Token::from(uuid)));
    }

    const PAYLOAD_LIMIT: usize = 1_048_576;

    fn request_with_body(content_type: &str, body: Vec<u8>) -> ServiceRequest {
        TestRequest::post()
            .app_data(PayloadConfig::new(PAYLOAD_LIMIT))
            .insert_header(("Content-Type", content_type))
            .set_payload(body)
            .to_srv_request()
    }

    async fn remaining_body(request: &mut ServiceRequest) -> Vec<u8> {
        let mut payload = request.take_payload();
        let mut body = Vec::new();
        while let Some(chunk) = payload.next().await {
            body.extend_from_slice(&chunk.unwrap());
        }
        body
    }

    #[actix_web::test]
    async fn extract_permissions_leaves_multipart_body_over_payload_limit_unread() {
        let body = vec![0xAB; 2 * PAYLOAD_LIMIT];
        let mut request = request_with_body("multipart/form-data; boundary=x", body.clone());

        let permissions = extract_permissions(&mut request).await.unwrap();

        assert!(permissions.is_empty());
        assert_eq!(remaining_body(&mut request).await, body);
    }

    #[actix_web::test]
    async fn extract_permissions_leaves_body_without_content_type_unread() {
        let body = vec![0xAB; 2 * PAYLOAD_LIMIT];
        let mut request = TestRequest::post()
            .app_data(PayloadConfig::new(PAYLOAD_LIMIT))
            .set_payload(body.clone())
            .to_srv_request();

        let permissions = extract_permissions(&mut request).await.unwrap();

        assert!(permissions.is_empty());
        assert_eq!(remaining_body(&mut request).await, body);
    }

    #[actix_web::test]
    async fn extract_permissions_restores_json_body_without_token() {
        let body = br#"{"title":"x"}"#.to_vec();
        let mut request = request_with_body("application/json", body.clone());

        let permissions = extract_permissions(&mut request).await.unwrap();

        assert!(permissions.is_empty());
        assert_eq!(remaining_body(&mut request).await, body);
    }

    #[actix_web::test]
    async fn extract_permissions_reads_json_body_for_a_token() {
        let body = vec![b' '; 2 * PAYLOAD_LIMIT];
        let mut request = request_with_body("application/merge-patch+json", body);

        let error = extract_permissions(&mut request)
            .await
            .expect_err("the JSON body is buffered and exceeds the payload limit");

        assert_eq!(
            error.as_response_error().status_code(),
            StatusCode::PAYLOAD_TOO_LARGE
        );
    }

    #[test]
    fn has_json_body_accepts_json_content_types() {
        for content_type in [
            "application/json",
            "application/json; charset=utf-8",
            "application/merge-patch+json",
        ] {
            let request = TestRequest::default()
                .insert_header(("Content-Type", content_type))
                .to_http_request();
            assert!(has_json_body(&request), "{content_type}");
        }
    }

    #[test]
    fn has_json_body_rejects_other_content_types() {
        for content_type in [
            "multipart/form-data; boundary=x",
            "application/x-www-form-urlencoded",
            "text/plain",
        ] {
            let request = TestRequest::default()
                .insert_header(("Content-Type", content_type))
                .to_http_request();
            assert!(!has_json_body(&request), "{content_type}");
        }
        assert!(!has_json_body(&TestRequest::default().to_http_request()));
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

    #[actix_web::test]
    async fn init_tracer_provider_builds_inside_async_runtime() {
        let provider = init_tracer_provider("test-key")
            .expect("the Honeycomb exporter builds inside the actix runtime");

        provider
            .shutdown()
            .expect("the provider shuts down cleanly");
    }

    #[test]
    fn tracing_spans_are_exported_with_service_name() {
        let exporter = RecordingExporter::default();
        let provider = tracer_provider_builder()
            .with_simple_exporter(exporter.clone())
            .build();
        let subscriber = tracing_subscriber::Registry::default()
            .with(tracing_opentelemetry::layer().with_tracer(provider.tracer(SERVICE_NAME)));

        tracing::subscriber::with_default(subscriber, || {
            tracing::info_span!("telemetry_test_span").in_scope(|| {});
        });
        provider.force_flush().expect("the provider flushes");

        let span_names: Vec<_> = exporter
            .spans
            .lock()
            .unwrap()
            .iter()
            .map(|span| span.name.clone())
            .collect();
        assert_eq!(span_names, ["telemetry_test_span"]);
        let service_name = exporter
            .resource
            .lock()
            .unwrap()
            .as_ref()
            .expect("the provider hands its resource to the exporter")
            .get(&opentelemetry::Key::from_static_str("service.name"));
        assert_eq!(service_name, Some(opentelemetry::Value::from(SERVICE_NAME)));
    }

    #[test]
    fn tracer_provider_guard_flushes_batched_spans_on_drop() {
        let exporter = RecordingExporter::default();
        let provider = tracer_provider_builder()
            .with_batch_exporter(exporter.clone())
            .build();
        let guard = TracerProviderGuard(provider.clone());
        let subscriber = tracing_subscriber::Registry::default()
            .with(tracing_opentelemetry::layer().with_tracer(provider.tracer(SERVICE_NAME)));

        tracing::subscriber::with_default(subscriber, || {
            tracing::info_span!("queued_span").in_scope(|| {});
        });
        assert!(
            exporter.spans.lock().unwrap().is_empty(),
            "the batch processor holds the span until its next scheduled export"
        );

        drop(guard);

        let span_names: Vec<_> = exporter
            .spans
            .lock()
            .unwrap()
            .iter()
            .map(|span| span.name.clone())
            .collect();
        assert_eq!(span_names, ["queued_span"]);
    }

    /// Span exporter that keeps every exported span and the resource the provider assigns it.
    #[derive(Debug, Clone, Default)]
    struct RecordingExporter {
        spans: Arc<std::sync::Mutex<Vec<opentelemetry_sdk::trace::SpanData>>>,
        resource: Arc<std::sync::Mutex<Option<Resource>>>,
    }

    impl opentelemetry_sdk::trace::SpanExporter for RecordingExporter {
        fn export(
            &self,
            mut batch: Vec<opentelemetry_sdk::trace::SpanData>,
        ) -> impl std::future::Future<Output = opentelemetry_sdk::error::OTelSdkResult> + Send
        {
            self.spans.lock().unwrap().append(&mut batch);
            std::future::ready(Ok(()))
        }

        fn set_resource(&mut self, resource: &Resource) {
            *self.resource.lock().unwrap() = Some(resource.clone());
        }
    }

    fn cache_with_stale_grant(token: &str) -> TokenPermissionsCache {
        let cache = TokenPermissionsCache::new();
        cache.set(
            token.to_string(),
            HashSet::from(["can_add_item".to_string()]),
        );
        cache
    }

    async fn revoked_lookup(_token: String) -> Result<HashSet<String>, Error> {
        Ok(HashSet::new())
    }

    #[actix_web::test]
    async fn permissions_for_method_ignores_cache_for_mutating_methods() {
        for method in [
            actix_web::http::Method::POST,
            actix_web::http::Method::PATCH,
            actix_web::http::Method::PUT,
            actix_web::http::Method::DELETE,
        ] {
            let cache = cache_with_stale_grant("tok");

            let permissions =
                permissions_for_method(&cache, &method, "tok".to_string(), revoked_lookup)
                    .await
                    .unwrap();

            assert!(permissions.is_empty(), "{method} used the cached grant");
            assert_eq!(
                cache.get("tok"),
                Some(HashSet::new()),
                "{method} did not refresh the cache"
            );
        }
    }

    #[actix_web::test]
    async fn permissions_for_method_serves_safe_methods_from_cache() {
        for method in [
            actix_web::http::Method::GET,
            actix_web::http::Method::HEAD,
            actix_web::http::Method::OPTIONS,
        ] {
            let cache = cache_with_stale_grant("tok");

            let permissions =
                permissions_for_method(&cache, &method, "tok".to_string(), |_| async {
                    panic!("{method} looked up a cached token")
                })
                .await
                .unwrap();

            assert_eq!(permissions, HashSet::from(["can_add_item".to_string()]));
        }
    }

    #[actix_web::test]
    async fn permissions_for_method_looks_up_and_caches_on_safe_method_miss() {
        let cache = TokenPermissionsCache::new();
        let granted = HashSet::from(["can_add_item".to_string()]);

        let permissions = permissions_for_method(
            &cache,
            &actix_web::http::Method::GET,
            "tok".to_string(),
            |_| async { Ok(HashSet::from(["can_add_item".to_string()])) },
        )
        .await
        .unwrap();

        assert_eq!(permissions, granted);
        assert_eq!(cache.get("tok"), Some(granted));
    }

    #[actix_web::test]
    async fn permissions_for_method_propagates_lookup_errors_without_caching() {
        let cache = TokenPermissionsCache::new();

        let result = permissions_for_method(
            &cache,
            &actix_web::http::Method::POST,
            "tok".to_string(),
            |_| async { Err(error::ErrorInternalServerError("db down")) },
        )
        .await;

        assert!(result.is_err());
        assert!(cache.get("tok").is_none());
    }

    #[test]
    fn bearer_token_from_headers_returns_none_for_malformed_uuid() {
        let request = TestRequest::default()
            .insert_header(("Authorization", "Bearer not-a-uuid"))
            .to_http_request();

        assert_eq!(bearer_token_from_headers(&request), None);
    }
}
