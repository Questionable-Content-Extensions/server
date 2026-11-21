use actix_web::web;

mod v2;
mod v3;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/v2").configure(v2::configure_v2));
    cfg.service(web::scope("/v3").configure(v3::configure_v3));
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::{App, test};

    #[actix_web::test]
    async fn unversioned_api_paths_are_not_served() {
        let app =
            test::init_service(App::new().service(web::scope("/api").configure(configure))).await;

        for (method, path) in [
            ("GET", "/api/comicdata/"),
            ("GET", "/api/comicdata/1"),
            ("GET", "/api/comicdata/excluded"),
            ("POST", "/api/comicdata/additem"),
            ("POST", "/api/comicdata/settitle"),
            ("GET", "/api/itemdata/"),
            ("GET", "/api/itemdata/1"),
            ("POST", "/api/itemdata/setproperty"),
            ("GET", "/api/log/"),
        ] {
            let request = match method {
                "GET" => test::TestRequest::get(),
                _ => test::TestRequest::post(),
            }
            .uri(path)
            .to_request();

            let response = test::call_service(&app, request).await;
            assert_eq!(
                response.status(),
                StatusCode::NOT_FOUND,
                "{method} {path} should not be routed"
            );
        }
    }
}
