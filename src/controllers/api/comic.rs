use crate::controllers::api::comic::editor_data::fetch_editor_data_for_comic;
use crate::controllers::api::comic::navigation_data::{
    fetch_all_item_navigation_data, fetch_comic_item_navigation_data,
};
use crate::database::models::{Comic as DatabaseComic, Item as DatabaseItem, News as DatabaseNews};
use crate::database::DbPool;
use crate::models::{
    token_permissions, Comic, ComicId, ComicList, Exclusion, Inclusion, ItemColor, ItemId,
    ItemType, Token,
};
use crate::util::{ensure_is_authorized, log_action, NewsUpdater};
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::{AuthDetails, PermissionsCheck};
use chrono::{DateTime, TimeZone, Utc};
use futures::TryStreamExt;
use log::info;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};

mod editor_data;
pub(crate) mod navigation_data;

mod add_item;
mod remove_item;
mod set_title;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(all)))
        .service(web::resource("/excluded").route(web::get().to(excluded)))
        .service(web::resource("/additem").route(web::post().to(add_item::add_item)))
        .service(web::resource("/removeitem").route(web::post().to(remove_item::remove_item)))
        .service(web::resource("/settitle").route(web::post().to(set_title::set_title)))
        .service(web::resource("/settagline").route(web::post().to(set_tagline)))
        .service(web::resource("/setpublishdate").route(web::post().to(set_publish_date)))
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

    let (is_guest_comic, is_non_canon) = match query.exclude {
        None => (None, None),
        Some(Exclusion::Guest) => (Some(false), None),
        Some(Exclusion::NonCanon) => (None, Some(false)),
    };

    let comic: Option<DatabaseComic> = sqlx::query_as!(
        DatabaseComic,
        r#"
		SELECT * FROM `comic`
		WHERE `id` = ?
	"#,
        comic_id.into_inner()
    )
    .fetch_optional(&mut conn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    let previous: Option<i16> = sqlx::query_scalar!(
        r#"
			SELECT id
			FROM `comic`
			WHERE id < ?
				AND (? is NULL OR `isGuestComic` = ?)
				AND (? is NULL OR `isNonCanon` = ?)
			ORDER BY id DESC
		"#,
        comic_id.into_inner(),
        is_guest_comic,
        is_guest_comic,
        is_non_canon,
        is_non_canon,
    )
    .fetch_optional(&mut conn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    let next: Option<i16> = sqlx::query_scalar!(
        r#"
			SELECT id
			FROM `comic`
			WHERE id > ?
				AND (? is NULL OR `isGuestComic` = ?)
				AND (? is NULL OR `isNonCanon` = ?)
			ORDER BY id ASC
		"#,
        comic_id.into_inner(),
        is_guest_comic,
        is_guest_comic,
        is_non_canon,
        is_non_canon,
    )
    .fetch_optional(&mut conn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    let news: Option<DatabaseNews> = if comic.is_some() {
        news_updater.check_for(comic_id).await;

        sqlx::query_as!(
            DatabaseNews,
            r#"
				SELECT * FROM `news`
				WHERE `comic` = ?
			"#,
            comic_id.into_inner()
        )
        .fetch_optional(&mut conn)
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

    let mut items: BTreeMap<ItemId, DatabaseItem> = sqlx::query_as!(
        DatabaseItem,
        r#"
            SELECT i.*
            FROM items i
            JOIN occurences o ON o.items_id = i.id
            WHERE o.comic_id = ?
        "#,
        comic_id.into_inner(),
    )
    .fetch(&mut conn)
    .map_ok(|i| (i.id.into(), i))
    .try_collect()
    .await
    .map_err(error::ErrorInternalServerError)?;

    let (comic_navigation_items, all_navigation_items) =
        if items.is_empty() && !matches!(query.include, Some(Inclusion::All)) {
            (vec![], vec![])
        } else if let Some(Inclusion::All) = query.include {
            let mut all_items: BTreeMap<ItemId, DatabaseItem> = sqlx::query_as!(
                DatabaseItem,
                r#"
				SELECT *
				FROM items
			"#,
            )
            .fetch(&mut conn)
            .map_ok(|i| (i.id.into(), i))
            .try_collect()
            .await
            .map_err(error::ErrorInternalServerError)?;

            let mut all_navigation_items =
                fetch_all_item_navigation_data(&mut conn, comic_id, is_guest_comic, is_non_canon)
                    .await?;
            let mut navigation_items_in_comic = vec![];
            let mut i = 0;
            while i < all_navigation_items.len() {
                let element = &mut all_navigation_items[i];
                if items.get(&element.id).is_some() {
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
            let mut navigation_items_in_comic =
                fetch_comic_item_navigation_data(&mut conn, comic_id, is_guest_comic, is_non_canon)
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
            publish_date: comic.publishDate.map(|nd| DateTime::from_utc(nd, Utc)),
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

async fn set_tagline(
    pool: web::Data<DbPool>,
    request: web::Json<SetTaglineBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    ensure_is_authorized(&auth, token_permissions::CAN_CHANGE_COMIC_DATA)
        .map_err(error::ErrorForbidden)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    ensure_comic_exists(&mut *transaction, request.comic_id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let old_tagline = sqlx::query_scalar!(
        r#"
            SELECT tagline FROM `comic` WHERE id = ?
        "#,
        request.comic_id.into_inner()
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?;

    sqlx::query!(
        r#"
            UPDATE `comic`
            SET tagline = ?
            WHERE
                id = ?
        "#,
        request.tagline,
        request.comic_id.into_inner(),
    )
    .execute(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?;

    match old_tagline {
        Some(old_tagline) if !old_tagline.is_empty() => {
            log_action(
                &mut *transaction,
                request.token,
                format!(
                    "Changed tagline on comic #{} from \"{}\" to \"{}\"",
                    request.comic_id, old_tagline, request.tagline
                ),
            )
            .await
            .map_err(error::ErrorInternalServerError)?;
        }
        _ => {
            log_action(
                &mut *transaction,
                request.token,
                format!(
                    "Set tagline on comic #{} to \"{}\"",
                    request.comic_id, request.tagline
                ),
            )
            .await
            .map_err(error::ErrorInternalServerError)?;
        }
    }

    transaction
        .commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body("Tagline set or updated for comic"))
}

async fn set_publish_date(
    pool: web::Data<DbPool>,
    request: web::Json<SetPublishDateBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    ensure_is_authorized(&auth, token_permissions::CAN_CHANGE_COMIC_DATA)
        .map_err(error::ErrorForbidden)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    ensure_comic_exists(&mut *transaction, request.comic_id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let old_publish_date = sqlx::query_scalar!(
        r#"
            SELECT publishDate FROM `comic` WHERE id = ?
        "#,
        request.comic_id.into_inner()
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?;

    sqlx::query!(
        r#"
            UPDATE `comic`
            SET
                publishDate = ?,
                isAccuratePublishDate = ?
            WHERE
                id = ?
        "#,
        request.publish_date.naive_utc(),
        request.is_accurate_publish_date,
        request.comic_id.into_inner(),
    )
    .execute(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?;

    if let Some(old_publish_date) = old_publish_date {
        log_action(
            &mut *transaction,
            request.token,
            format!(
                "Changed publish date on comic #{} from \"{}\" to \"{}\"",
                request.comic_id,
                Utc.from_utc_datetime(&old_publish_date)
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                request
                    .publish_date
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            ),
        )
        .await
        .map_err(error::ErrorInternalServerError)?;
    } else {
        log_action(
            &mut *transaction,
            request.token,
            format!(
                "Set publish date on comic #{} to \"{}\"",
                request.comic_id,
                request
                    .publish_date
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            ),
        )
        .await
        .map_err(error::ErrorInternalServerError)?;
    }

    Ok(HttpResponse::Ok().finish())
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

    ensure_comic_exists(&mut *transaction, request.comic_id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let (true_value_log_text, false_value_log_text, sql_future) = match flag {
        FlagType::IsGuestComic => (
            "to be a guest comic",
            "to be a Jeph comic",
            sqlx::query!(
                r#"
                    UPDATE `comic`
                    SET
                        isGuestComic = ?
                    WHERE
                        id = ?
                "#,
                request.flag_value,
                request.comic_id.into_inner(),
            )
            .execute(&mut *transaction),
        ),
        FlagType::IsNonCanon => (
            "to be non-canon",
            "to be canon",
            sqlx::query!(
                r#"
                    UPDATE `comic`
                    SET
                        isNonCanon = ?
                    WHERE
                        id = ?
                "#,
                request.flag_value,
                request.comic_id.into_inner(),
            )
            .execute(&mut *transaction),
        ),
        FlagType::HasNoCast => (
            "to have no cast",
            "to have cast",
            sqlx::query!(
                r#"
                    UPDATE `comic`
                    SET
                        HasNoCast = ?
                    WHERE
                        id = ?
                "#,
                request.flag_value,
                request.comic_id.into_inner(),
            )
            .execute(&mut *transaction),
        ),
        FlagType::HasNoLocation => (
            "to have no locations",
            "to have locations",
            sqlx::query!(
                r#"
                    UPDATE `comic`
                    SET
                        HasNoLocation = ?
                    WHERE
                        id = ?
                "#,
                request.flag_value,
                request.comic_id.into_inner(),
            )
            .execute(&mut *transaction),
        ),
        FlagType::HasNoStoryline => (
            "to have no storylines",
            "to have storylines",
            sqlx::query!(
                r#"
                    UPDATE `comic`
                    SET
                        HasNoStoryline = ?
                    WHERE
                        id = ?
                "#,
                request.flag_value,
                request.comic_id.into_inner(),
            )
            .execute(&mut *transaction),
        ),
        FlagType::HasNoTitle => (
            "to have no title",
            "to have a title",
            sqlx::query!(
                r#"
                    UPDATE `comic`
                    SET
                        HasNoTitle = ?
                    WHERE
                        id = ?
                "#,
                request.flag_value,
                request.comic_id.into_inner(),
            )
            .execute(&mut *transaction),
        ),
        FlagType::HasNoTagline => (
            "to have no tagline",
            "to have a tagline",
            sqlx::query!(
                r#"
                    UPDATE `comic`
                    SET
                        HasNoTagline = ?
                    WHERE
                        id = ?
                "#,
                request.flag_value,
                request.comic_id.into_inner(),
            )
            .execute(&mut *transaction),
        ),
    };

    sql_future.await.map_err(error::ErrorInternalServerError)?;

    log_action(
        &mut *transaction,
        request.token,
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
    items: &mut BTreeMap<ItemId, DatabaseItem>,
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
            .remove(&navigation_item.id)
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
    let comics: Vec<ComicList> = sqlx::query_as!(
        DatabaseComic,
        r#"
			SELECT * FROM `comic`
			WHERE (? is NULL OR `isGuestComic` = ?)
				AND (? is NULL OR `isNonCanon` = ?)
			ORDER BY id ASC
		"#,
        is_guest_comic,
        is_guest_comic,
        is_non_canon,
        is_non_canon
    )
    .fetch(&**pool)
    .map_ok(From::from)
    .try_collect()
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(comics)
}

#[inline]
async fn ensure_comic_exists<'e, 'c: 'e, E>(executor: E, comic_id: ComicId) -> sqlx::Result<()>
where
    E: 'e + sqlx::Executor<'c, Database = sqlx::MySql>,
{
    sqlx::query!(
        r#"
            INSERT IGNORE INTO `comic`
                (id)
            VALUES
                (?)
        "#,
        comic_id.into_inner(),
    )
    .execute(executor)
    .await?;

    Ok(())
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
struct SetTaglineBody {
    token: Token,
    comic_id: ComicId,
    tagline: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetPublishDateBody {
    token: Token,
    comic_id: ComicId,
    publish_date: DateTime<Utc>,
    is_accurate_publish_date: bool,
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
