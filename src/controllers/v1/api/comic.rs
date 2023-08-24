use actix_web::web;

use crate::controllers::v2::api::comic::remove_item;

pub(crate) mod navigation_data;

pub(in crate::controllers) mod add_item;
pub(in crate::controllers) mod all;
pub(in crate::controllers) mod by_id;
pub(in crate::controllers) mod editor_data;
pub(in crate::controllers) mod set_flags;
pub(in crate::controllers) mod set_publish_date;
pub(in crate::controllers) mod set_tagline;
pub(in crate::controllers) mod set_title;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(all::all)))
        .service(web::resource("/excluded").route(web::get().to(all::excluded)))
        .service(web::resource("/additem").route(web::post().to(add_item::add_item)))
        .service(web::resource("/removeitem").route(web::post().to(remove_item::remove_item)))
        .service(web::resource("/settitle").route(web::post().to(set_title::set_title)))
        .service(web::resource("/settagline").route(web::post().to(set_tagline::set_tagline)))
        .service(
            web::resource("/setpublishdate")
                .route(web::post().to(set_publish_date::set_publish_date)),
        )
        .service(web::resource("/setguest").route(web::post().to(set_flags::set_guest)))
        .service(web::resource("/setnoncanon").route(web::post().to(set_flags::set_non_canon)))
        .service(web::resource("/setnocast").route(web::post().to(set_flags::set_no_cast)))
        .service(web::resource("/setnolocation").route(web::post().to(set_flags::set_no_location)))
        .service(
            web::resource("/setnostoryline").route(web::post().to(set_flags::set_no_storyline)),
        )
        .service(web::resource("/setnotitle").route(web::post().to(set_flags::set_no_title)))
        .service(web::resource("/setnotagline").route(web::post().to(set_flags::set_no_tagline)))
        .service(web::resource("/{comicId}").route(web::get().to(by_id::by_id)));
}
