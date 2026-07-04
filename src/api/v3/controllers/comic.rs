use actix_web::web;

pub(super) mod active_storylines;
pub(super) mod navigation_data;

mod add_advance_comic;
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
        .service(patch_comic::patch_comic)
        .service(add_advance_comic::add_advance_comic)
        .service(add_advance_comic::list_advance_comics)
        .service(add_advance_comic::run_comic_updater)
        .service(by_id::by_id);
}
