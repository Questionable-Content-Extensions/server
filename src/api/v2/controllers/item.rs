use actix_web::web;

mod all;
mod by_id;
mod image;
mod image_upload;
mod patch_item;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(all::all)))
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
        .service(web::resource("{itemId}/comics").route(web::get().to(by_id::comics)))
        .service(web::resource("{itemId}/comics/random").route(web::get().to(by_id::random_comic)))
        .service(web::resource("{itemId}/friends").route(web::get().to(by_id::friends)))
        .service(web::resource("{itemId}/locations").route(web::get().to(by_id::locations)))
        .service(
            web::resource("{itemId}/images")
                .route(web::get().to(image::images))
                .route(web::post().to(image_upload::image_upload)),
        )
        .service(
            web::resource("{itemId}/images/primary").route(web::post().to(image::set_primary)),
        );
}
