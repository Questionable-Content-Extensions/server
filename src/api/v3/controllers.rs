use actix_web::web;

mod comic;
mod item;
mod log;
mod stats;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/comicdata").configure(comic::configure));
    cfg.service(web::scope("/itemdata").configure(item::configure));
    cfg.service(web::scope("/log").configure(log::configure));
    cfg.service(web::scope("/stats").configure(stats::configure));
}
