use actix_web::web;

mod v1;
mod v2;
mod v3;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/v2").configure(v2::configure_v2));
    cfg.service(web::scope("/v3").configure(v3::configure_v3));
    cfg.service(web::scope("").configure(v1::configure_v1));
}
