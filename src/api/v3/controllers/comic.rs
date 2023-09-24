use actix_web::web;

pub(super) mod navigation_data;

mod add_item;
mod all;
mod by_id;
mod editor_data;
mod patch_comic;
mod remove_item;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(all::all)))
        .service(web::resource("/excluded").route(web::get().to(all::excluded)))
        .service(web::resource("/containing-items").route(web::get().to(all::containing_items)))
        .service(web::resource("/additem").route(web::post().to(add_item::add_item)))
        .service(web::resource("/additems").route(web::post().to(add_item::add_items)))
        .service(web::resource("/removeitem").route(web::post().to(remove_item::remove_item)))
        .service(
            web::resource("/{comicId}")
                .route(web::get().to(by_id::by_id))
                .route(web::patch().to(patch_comic::patch_comic)),
        );
}
