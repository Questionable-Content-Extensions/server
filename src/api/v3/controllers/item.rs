use actix_web::web;

mod all;
mod by_id;
mod image;
mod image_upload;
mod patch_item;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(all::all)
        .service(web::resource("image/{imageId}").route(web::get().to(image::image)))
        .service(image::delete)
        .service(by_id::by_id)
        .service(patch_item::patch_item)
        .service(by_id::comics)
        .service(by_id::random_comic)
        .service(by_id::friends)
        .service(by_id::locations)
        .service(image::images)
        .service(web::resource("{itemId}/images").route(web::post().to(image_upload::image_upload)))
        .service(image::set_primary);
}
