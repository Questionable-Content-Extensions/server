use actix_web::web;

pub(in crate::controllers) mod comic;
pub(in crate::controllers) mod item;
pub(in crate::controllers) mod log;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/comicdata").configure(comic::configure));
    cfg.service(web::scope("/itemdata").configure(item::configure));
    cfg.service(web::scope("/log").configure(log::configure));

    // For legacy reasons, the endpoints above must live at the root of `/api`. Thus, any newer versions
    // must be mounted as children of v1, here:
    cfg.service(web::scope("/v2").configure(crate::controllers::v2::api::configure));
}
