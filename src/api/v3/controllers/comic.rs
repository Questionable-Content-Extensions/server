use actix_web::web;

pub(super) mod navigation_data;

mod add_item;
mod all;
mod by_id;
mod editor_data;
mod patch_comic;
mod remove_item;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(all::all)
        .service(all::excluded)
        .service(all::containing_items)
        .service(add_item::add_item)
        .service(add_item::add_items)
        .service(remove_item::remove_item)
        .service(by_id::by_id)
        .service(patch_comic::patch_comic);
}
