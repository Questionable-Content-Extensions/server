use actix_web::body::{BoxBody, EitherBody, MessageBody};
use actix_web::dev::ServiceResponse;
use actix_web::http::header::{self, HeaderValue};
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use tracing::error;

/// Body sent in place of every 5xx response's own body.
pub const SERVER_ERROR_MESSAGE: &str = "Internal server error";

/// Middleware that logs the error behind each 5xx response and replaces the response body with
/// [`SERVER_ERROR_MESSAGE`], so database and other internal error text never reaches the client.
///
/// The error stays attached to the response, so outer middleware such as `TracingLogger` still
/// records it.
pub fn hide_server_error_details<B>() -> ErrorHandlers<B>
where
    B: MessageBody + 'static,
{
    ErrorHandlers::new().default_handler_server(|response| Ok(replace_server_error_body(response)))
}

fn replace_server_error_body<B>(response: ServiceResponse<B>) -> ErrorHandlerResponse<B> {
    let status = response.status().as_u16();
    if let Some(e) = response.response().error() {
        error!(status, "Server error: {e}");
    } else {
        error!(status, "Server error without an error value");
    }

    let response = response.map_body(|head, _| {
        head.headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        head.headers.remove(header::CONTENT_LENGTH);
        EitherBody::right(BoxBody::new(SERVER_ERROR_MESSAGE))
    });

    ErrorHandlerResponse::Response(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::{App, HttpResponse, error, test, web};
    use std::io;
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::layer::SubscriberExt;

    /// Held by every test that reaches `replace_server_error_body`'s log callsite. Tracing can
    /// cache that callsite's interest as `never` if a thread without a subscriber registers it
    /// while `server_error_is_logged` installs its thread-local one.
    static LOG_CALLSITE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    const DATABASE_ERROR: &str = "Cannot add or update a child row: a foreign key constraint \
        fails (`dphoenix_qcnav`.`LogEntry`, CONSTRAINT `fk_user_token`) for user 'dphoenix_qc'";

    async fn database_error() -> actix_web::Result<HttpResponse> {
        Err(error::ErrorInternalServerError(DATABASE_ERROR))
    }

    async fn unavailable() -> actix_web::Result<HttpResponse> {
        Err(error::ErrorServiceUnavailable(
            "pool timed out for 'dphoenix_qc'",
        ))
    }

    async fn plain_server_error() -> HttpResponse {
        HttpResponse::InternalServerError().body("raw detail")
    }

    async fn forbidden() -> actix_web::Result<HttpResponse> {
        Err(error::ErrorForbidden(
            "Invalid token or insufficient permissions",
        ))
    }

    macro_rules! test_app {
        () => {
            test::init_service(
                App::new()
                    .wrap(hide_server_error_details())
                    .route("/db", web::get().to(database_error))
                    .route("/unavailable", web::get().to(unavailable))
                    .route("/plain", web::get().to(plain_server_error))
                    .route("/forbidden", web::get().to(forbidden)),
            )
            .await
        };
    }

    #[actix_web::test]
    async fn internal_server_error_body_is_replaced() {
        let _log_callsite = LOG_CALLSITE.lock().await;
        let app = test_app!();

        let response =
            test::call_service(&app, test::TestRequest::get().uri("/db").to_request()).await;

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/plain; charset=utf-8"
        );
        let body = test::read_body(response).await;
        assert_eq!(body, SERVER_ERROR_MESSAGE);
    }

    #[actix_web::test]
    async fn every_server_error_status_is_replaced() {
        let _log_callsite = LOG_CALLSITE.lock().await;
        let app = test_app!();

        for (uri, status) in [
            ("/unavailable", StatusCode::SERVICE_UNAVAILABLE),
            ("/plain", StatusCode::INTERNAL_SERVER_ERROR),
        ] {
            let response =
                test::call_service(&app, test::TestRequest::get().uri(uri).to_request()).await;

            assert_eq!(response.status(), status, "{uri}");
            let body = test::read_body(response).await;
            assert_eq!(body, SERVER_ERROR_MESSAGE, "{uri}");
        }
    }

    #[actix_web::test]
    async fn client_error_body_is_kept() {
        let app = test_app!();

        let response = test::call_service(
            &app,
            test::TestRequest::get().uri("/forbidden").to_request(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body = test::read_body(response).await;
        assert_eq!(body, "Invalid token or insufficient permissions");
    }

    #[actix_web::test]
    async fn replaced_response_keeps_its_error_for_outer_middleware() {
        let _log_callsite = LOG_CALLSITE.lock().await;
        let app = test_app!();

        let response =
            test::call_service(&app, test::TestRequest::get().uri("/db").to_request()).await;

        let error = response
            .response()
            .error()
            .expect("the handler's error stays attached");
        assert_eq!(error.to_string(), DATABASE_ERROR);
    }

    #[actix_web::test]
    async fn server_error_is_logged() {
        let _log_callsite = LOG_CALLSITE.lock().await;
        let log = SharedLog::default();
        let subscriber = tracing_subscriber::Registry::default().with(
            tracing_subscriber::fmt::Layer::default()
                .with_ansi(false)
                .with_writer({
                    let log = log.clone();
                    move || log.clone()
                }),
        );
        let _guard = tracing::subscriber::set_default(subscriber);
        let app = test_app!();

        test::call_service(&app, test::TestRequest::get().uri("/db").to_request()).await;

        let output = String::from_utf8(log.0.lock().unwrap().clone()).unwrap();
        assert!(output.contains("ERROR"), "{output}");
        assert!(output.contains(DATABASE_ERROR), "{output}");
        assert!(output.contains("status=500"), "{output}");
    }

    /// Writer that appends everything written to it to a shared buffer.
    #[derive(Debug, Clone, Default)]
    struct SharedLog(Arc<Mutex<Vec<u8>>>);

    impl io::Write for SharedLog {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
}
