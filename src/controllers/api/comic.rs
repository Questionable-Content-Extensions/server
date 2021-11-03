use crate::controllers::api::comic::editor_data::fetch_editor_data_for_comic;
use crate::controllers::api::comic::navigation_data::{
    fetch_all_item_navigation_data, fetch_comic_item_navigation_data,
};
use crate::models::{Comic, ComicId, ComicList, Exclusion, Inclusion, ItemColor, ItemType, Token};
use crate::util::{ensure_is_authorized, NewsUpdater};
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::{AuthDetails, PermissionsCheck};
use chrono::{TimeZone, Utc};
use database::models::{
    Comic as DatabaseComic, Item as DatabaseItem, LogEntry, News as DatabaseNews,
};
use database::DbPool;
use log::info;
use serde::Deserialize;
use shared::token_permissions;
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};

mod editor_data;
pub(crate) mod navigation_data;

mod add_item;
mod remove_item;
mod set_publish_date;
mod set_tagline;
mod set_title;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(all)))
        .service(web::resource("/excluded").route(web::get().to(excluded)))
        .service(web::resource("/additem").route(web::post().to(add_item::add_item)))
        .service(web::resource("/removeitem").route(web::post().to(remove_item::remove_item)))
        .service(web::resource("/settitle").route(web::post().to(set_title::set_title)))
        .service(web::resource("/settagline").route(web::post().to(set_tagline::set_tagline)))
        .service(
            web::resource("/setpublishdate")
                .route(web::post().to(set_publish_date::set_publish_date)),
        )
        .service(web::resource("/setguest").route(web::post().to(set_guest)))
        .service(web::resource("/setnoncanon").route(web::post().to(set_non_canon)))
        .service(web::resource("/setnocast").route(web::post().to(set_no_cast)))
        .service(web::resource("/setnolocation").route(web::post().to(set_no_location)))
        .service(web::resource("/setnostoryline").route(web::post().to(set_no_storyline)))
        .service(web::resource("/setnotitle").route(web::post().to(set_no_title)))
        .service(web::resource("/setnotagline").route(web::post().to(set_no_tagline)))
        .service(web::resource("/{comicId}").route(web::get().to(by_id)));
}

async fn all(pool: web::Data<DbPool>, query: web::Query<AllQuery>) -> Result<HttpResponse> {
    let (is_guest_comic, is_non_canon) = match query.exclude {
        None => (None, None),
        Some(Exclusion::Guest) => (Some(false), None),
        Some(Exclusion::NonCanon) => (None, Some(false)),
    };

    info!(
        "Requesting all comics (exclude guest comics: {}, exclude non-canon comics: {})",
        is_guest_comic.map_or(false, |v| !v),
        is_non_canon.map_or(false, |v| !v)
    );

    Ok(HttpResponse::Ok().json(fetch_comic_list(&pool, is_guest_comic, is_non_canon).await?))
}

async fn excluded(pool: web::Data<DbPool>, query: web::Query<AllQuery>) -> Result<HttpResponse> {
    let (is_guest_comic, is_non_canon) = match query.exclude {
        None => {
            return Err(error::ErrorBadRequest(
                "exclude parameter must be set to either `guest` or `non-canon`",
            ))
        }
        Some(Exclusion::Guest) => (Some(true), None),
        Some(Exclusion::NonCanon) => (None, Some(true)),
    };

    info!(
        "Requesting excluded {} comics",
        if is_guest_comic.is_some() {
            "guest"
        } else {
            "non-canon"
        }
    );

    Ok(HttpResponse::Ok().json(fetch_comic_list(&pool, is_guest_comic, is_non_canon).await?))
}

#[allow(clippy::too_many_lines)]
async fn by_id(
    pool: web::Data<DbPool>,
    news_updater: web::Data<NewsUpdater>,
    query: web::Query<ByIdQuery>,
    comic_id: web::Path<ComicId>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    let comic_id = comic_id.into_inner();

    let mut conn = pool
        .acquire()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let (include_guest_comics, include_non_canon_comics) = match query.exclude {
        None => (None, None),
        Some(Exclusion::Guest) => (Some(false), None),
        Some(Exclusion::NonCanon) => (None, Some(false)),
    };

    let comic = DatabaseComic::by_id(&mut conn, comic_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?;

    let previous = DatabaseComic::previous_id(
        &mut conn,
        comic_id.into_inner(),
        include_guest_comics,
        include_non_canon_comics,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    let next = DatabaseComic::next_id(
        &mut conn,
        comic_id.into_inner(),
        include_guest_comics,
        include_non_canon_comics,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    let news: Option<DatabaseNews> = if comic.is_some() {
        news_updater.check_for(comic_id).await;

        DatabaseNews::by_comic_id(&mut conn, comic_id.into_inner())
            .await
            .map_err(error::ErrorInternalServerError)?
    } else {
        None
    };

    let editor_data = if auth.has_permission(token_permissions::HAS_VALID_TOKEN) {
        Some(fetch_editor_data_for_comic(&mut conn, comic_id).await?)
    } else {
        None
    };

    let mut items =
        DatabaseItem::occurrences_in_comic_mapped_by_id(&mut conn, comic_id.into_inner())
            .await
            .map_err(error::ErrorInternalServerError)?;

    let (comic_navigation_items, all_navigation_items) =
        if items.is_empty() && !matches!(query.include, Some(Inclusion::All)) {
            (vec![], vec![])
        } else if let Some(Inclusion::All) = query.include {
            let mut all_items = DatabaseItem::all_mapped_by_id(&mut conn)
                .await
                .map_err(error::ErrorInternalServerError)?;

            let mut all_navigation_items = fetch_all_item_navigation_data(
                &mut conn,
                comic_id,
                include_guest_comics,
                include_non_canon_comics,
            )
            .await?;
            let mut navigation_items_in_comic = vec![];
            let mut i = 0;
            while i < all_navigation_items.len() {
                let element = &mut all_navigation_items[i];
                if items.get(&element.id.into_inner()).is_some() {
                    let element = all_navigation_items.remove(i);
                    navigation_items_in_comic.push(element);
                } else {
                    i += 1;
                }
            }

            transfer_item_data_to_navigation_item(&mut navigation_items_in_comic, &mut all_items)
                .map_err(error::ErrorInternalServerError)?;
            transfer_item_data_to_navigation_item(&mut all_navigation_items, &mut all_items)
                .map_err(error::ErrorInternalServerError)?;

            (navigation_items_in_comic, all_navigation_items)
        } else {
            let mut navigation_items_in_comic = fetch_comic_item_navigation_data(
                &mut conn,
                comic_id,
                include_guest_comics,
                include_non_canon_comics,
            )
            .await?;

            transfer_item_data_to_navigation_item(&mut navigation_items_in_comic, &mut items)
                .map_err(error::ErrorInternalServerError)?;

            (navigation_items_in_comic, vec![])
        };

    let comic = if let Some(comic) = comic {
        Comic {
            comic: comic_id,
            has_data: true,
            image_type: Some(comic.ImageType.into()),
            publish_date: comic.publishDate.map(|nd| Utc.from_utc_datetime(&nd)),
            is_accurate_publish_date: comic.isAccuratePublishDate != 0,
            title: Some(comic.title),
            tagline: comic.tagline,
            is_guest_comic: comic.isGuestComic != 0,
            is_non_canon: comic.isNonCanon != 0,
            has_no_cast: comic.HasNoCast != 0,
            has_no_location: comic.HasNoLocation != 0,
            has_no_storyline: comic.HasNoStoryline != 0,
            has_no_title: comic.HasNoTitle != 0,
            has_no_tagline: comic.HasNoTagline != 0,
            news: news.map(|n| n.news),
            previous: previous
                .map(TryInto::try_into)
                .transpose()
                .expect("database has valid comicIds"),
            next: next
                .map(TryInto::try_into)
                .transpose()
                .expect("database has valid comicIds"),
            editor_data,
            items: comic_navigation_items,
            all_items: all_navigation_items,
        }
    } else {
        Comic {
            comic: comic_id,
            has_data: false,
            image_type: None,
            publish_date: None,
            is_accurate_publish_date: false,
            title: None,
            tagline: None,
            is_guest_comic: false,
            is_non_canon: false,
            has_no_cast: false,
            has_no_location: false,
            has_no_storyline: false,
            has_no_title: false,
            has_no_tagline: false,
            news: None,
            previous: previous
                .map(TryInto::try_into)
                .transpose()
                .expect("database has valid comicIds"),
            next: next
                .map(TryInto::try_into)
                .transpose()
                .expect("database has valid comicIds"),
            editor_data,
            items: comic_navigation_items,
            all_items: all_navigation_items,
        }
    };

    Ok(HttpResponse::Ok().json(comic))
}

async fn set_guest(
    pool: web::Data<DbPool>,
    request: web::Json<SetFlagBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    set_flag(pool, request, auth, FlagType::IsGuestComic).await?;

    Ok(HttpResponse::Ok().body("Guest comic set or updated for comic"))
}

async fn set_non_canon(
    pool: web::Data<DbPool>,
    request: web::Json<SetFlagBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    set_flag(pool, request, auth, FlagType::IsNonCanon).await?;

    Ok(HttpResponse::Ok().body("Non-canon set or updated for comic"))
}

async fn set_no_cast(
    pool: web::Data<DbPool>,
    request: web::Json<SetFlagBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    set_flag(pool, request, auth, FlagType::HasNoCast).await?;

    Ok(HttpResponse::Ok().body("No cast set or updated for comic"))
}

async fn set_no_location(
    pool: web::Data<DbPool>,
    request: web::Json<SetFlagBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    set_flag(pool, request, auth, FlagType::HasNoLocation).await?;

    Ok(HttpResponse::Ok().body("No location set or updated for comic"))
}

async fn set_no_storyline(
    pool: web::Data<DbPool>,
    request: web::Json<SetFlagBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    set_flag(pool, request, auth, FlagType::HasNoStoryline).await?;

    Ok(HttpResponse::Ok().body("No storyline set or updated for comic"))
}

async fn set_no_title(
    pool: web::Data<DbPool>,
    request: web::Json<SetFlagBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    set_flag(pool, request, auth, FlagType::HasNoTitle).await?;

    Ok(HttpResponse::Ok().body("No title set or updated for comic"))
}

async fn set_no_tagline(
    pool: web::Data<DbPool>,
    request: web::Json<SetFlagBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    set_flag(pool, request, auth, FlagType::HasNoTagline).await?;

    Ok(HttpResponse::Ok().body("No tagline set or updated for comic"))
}

#[allow(clippy::too_many_lines)]
async fn set_flag(
    pool: web::Data<DbPool>,
    request: web::Json<SetFlagBody>,
    auth: AuthDetails,
    flag: FlagType,
) -> Result<()> {
    ensure_is_authorized(&auth, token_permissions::CAN_CHANGE_COMIC_DATA)
        .map_err(error::ErrorForbidden)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    DatabaseComic::ensure_exists_by_id(&mut *transaction, request.comic_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?;

    let (true_value_log_text, false_value_log_text, sql_result) = match flag {
        FlagType::IsGuestComic => (
            "to be a guest comic",
            "to be a Jeph comic",
            DatabaseComic::update_is_guest_comic_by_id(
                &mut *transaction,
                request.comic_id.into_inner(),
                request.flag_value,
            )
            .await,
        ),
        FlagType::IsNonCanon => (
            "to be non-canon",
            "to be canon",
            DatabaseComic::update_is_non_canon_by_id(
                &mut *transaction,
                request.comic_id.into_inner(),
                request.flag_value,
            )
            .await,
        ),
        FlagType::HasNoCast => (
            "to have no cast",
            "to have cast",
            DatabaseComic::update_has_no_cast_by_id(
                &mut *transaction,
                request.comic_id.into_inner(),
                request.flag_value,
            )
            .await,
        ),
        FlagType::HasNoLocation => (
            "to have no locations",
            "to have locations",
            DatabaseComic::update_has_no_location_by_id(
                &mut *transaction,
                request.comic_id.into_inner(),
                request.flag_value,
            )
            .await,
        ),
        FlagType::HasNoStoryline => (
            "to have no storylines",
            "to have storylines",
            DatabaseComic::update_has_no_storyline_by_id(
                &mut *transaction,
                request.comic_id.into_inner(),
                request.flag_value,
            )
            .await,
        ),
        FlagType::HasNoTitle => (
            "to have no title",
            "to have a title",
            DatabaseComic::update_has_no_title_by_id(
                &mut *transaction,
                request.comic_id.into_inner(),
                request.flag_value,
            )
            .await,
        ),
        FlagType::HasNoTagline => (
            "to have no tagline",
            "to have a tagline",
            DatabaseComic::update_has_no_tagline_by_id(
                &mut *transaction,
                request.comic_id.into_inner(),
                request.flag_value,
            )
            .await,
        ),
    };

    sql_result.map_err(error::ErrorInternalServerError)?;

    LogEntry::log_action(
        &mut *transaction,
        request.token.to_string(),
        format!(
            "Set comic #{} {}",
            request.comic_id,
            if request.flag_value {
                true_value_log_text
            } else {
                false_value_log_text
            }
        ),
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    transaction
        .commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(())
}

fn transfer_item_data_to_navigation_item(
    navigation_items: &mut Vec<crate::models::ItemNavigationData>,
    items: &mut BTreeMap<u16, DatabaseItem>,
) -> anyhow::Result<()> {
    for navigation_item in navigation_items {
        let DatabaseItem {
            id: _,
            shortName: short_name,
            name,
            r#type,
            Color_Red: color_red,
            Color_Green: color_green,
            Color_Blue: color_blue,
        } = items
            .remove(&navigation_item.id.into_inner())
            .expect("item data for navigation item");
        navigation_item.short_name = Some(short_name);
        navigation_item.name = Some(name);
        navigation_item.r#type = Some(ItemType::try_from(&*r#type)?);

        navigation_item.color = Some(ItemColor(color_red, color_green, color_blue));
    }

    Ok(())
}

async fn fetch_comic_list(
    pool: &DbPool,
    is_guest_comic: Option<bool>,
    is_non_canon: Option<bool>,
) -> Result<Vec<ComicList>> {
    let comics: Vec<ComicList> =
        DatabaseComic::all_with_mapping(&**pool, is_guest_comic, is_non_canon, From::from)
            .await
            .map_err(error::ErrorInternalServerError)?;

    Ok(comics)
}

#[derive(Debug, Deserialize)]
struct AllQuery {
    exclude: Option<Exclusion>,
}

#[derive(Debug, Deserialize)]
struct ExcludedQuery {
    exclusion: Option<Exclusion>,
}

#[derive(Debug, Deserialize)]
struct ByIdQuery {
    exclude: Option<Exclusion>,
    include: Option<Inclusion>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetFlagBody {
    token: Token,
    comic_id: ComicId,
    flag_value: bool,
}

pub enum FlagType {
    IsGuestComic,
    IsNonCanon,
    HasNoCast,
    HasNoLocation,
    HasNoStoryline,
    HasNoTitle,
    HasNoTagline,
}
