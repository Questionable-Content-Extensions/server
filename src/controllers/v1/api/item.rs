use actix_web::web;

pub(in crate::controllers) mod all;
pub(in crate::controllers) mod by_id;
pub(in crate::controllers) mod image_upload;
pub(in crate::controllers) mod set_property;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(all::all)))
        .service(web::resource("setproperty").route(web::post().to(set_property::set_property)))
        .service(web::resource("image/upload").route(web::post().to(image_upload::image_upload)))
        .service(web::resource("image/{imageId}").route(web::get().to(by_id::image)))
        .service(web::resource("friends/{itemId}").route(web::get().to(by_id::friends)))
        .service(web::resource("locations/{itemId}").route(web::get().to(by_id::locations)))
        .service(web::resource("{itemId}").route(web::get().to(by_id::by_id)))
        .service(web::resource("{itemId}/friends").route(web::get().to(by_id::friends)))
        .service(web::resource("{itemId}/locations").route(web::get().to(by_id::locations)))
        .service(web::resource("{itemId}/images").route(web::get().to(by_id::images)));
}
