use actix_web::web;

mod comic;

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/comicdata").configure(comic::configure));
}
