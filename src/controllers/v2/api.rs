use actix_web::web;

pub(in crate::controllers) mod comic;
pub(in crate::controllers) mod item;
pub(in crate::controllers) mod log;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/comicdata").configure(comic::configure));
    cfg.service(web::scope("/itemdata").configure(item::configure));
    cfg.service(web::scope("/log").configure(log::configure));
}
