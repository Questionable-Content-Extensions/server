use actix_web::web;

pub(in crate::controllers) use crate::controllers::v1::api::item::*;

pub(in crate::controllers) mod by_id;
pub(in crate::controllers) mod image;
pub(in crate::controllers) mod patch_item;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(all::all)))
        .service(web::resource("image/upload").route(web::post().to(image_upload::image_upload)))
        .service(
            web::resource("image/{imageId}")
                .route(web::get().to(image::image))
                .route(web::delete().to(image::delete)),
        )
        .service(
            web::resource("{itemId}")
                .route(web::get().to(by_id::by_id))
                .route(web::patch().to(patch_item::patch_item)),
        )
        .service(web::resource("{itemId}/friends").route(web::get().to(by_id::friends)))
        .service(web::resource("{itemId}/locations").route(web::get().to(by_id::locations)))
        .service(web::resource("{itemId}/images").route(web::get().to(image::images)))
        .service(
            web::resource("{itemId}/images/primary").route(web::post().to(image::set_primary)),
        );
}
