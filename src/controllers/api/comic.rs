use std::collections::BTreeMap;

use crate::controllers::api::comic::editor_data::fetch_editor_data_for_comic;
use crate::controllers::api::comic::navigation_data::{
    fetch_all_item_navigation_data, fetch_comic_item_navigation_data,
};
use crate::database::models::{Comic as DatabaseComic, Item, News as DatabaseNews};
use crate::database::DbPool;
use crate::models::{Comic, ComicList, Exclusion, Inclusion};
use crate::util::{is_token_valid, NewsUpdater};
use actix_web::{error, web, HttpResponse, Result};
use chrono::{DateTime, Utc};
use futures::{TryFutureExt, TryStreamExt};
use log::info;
use serde::Deserialize;

mod editor_data;
mod navigation_data;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(all)))
        .service(web::resource("/excluded").route(web::get().to(excluded)))
        .service(web::resource("/{comicId}").route(web::get().to(by_id)));
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
    token: Option<uuid::Uuid>,
}

async fn all(conn: web::Data<DbPool>, query: web::Query<AllQuery>) -> Result<HttpResponse> {
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

    Ok(HttpResponse::Ok().json(fetch_comic_list(&conn, is_guest_comic, is_non_canon).await?))
}

async fn excluded(conn: web::Data<DbPool>, query: web::Query<AllQuery>) -> Result<HttpResponse> {
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

    Ok(HttpResponse::Ok().json(fetch_comic_list(&conn, is_guest_comic, is_non_canon).await?))
}

#[allow(clippy::too_many_lines)]
async fn by_id(
    conn: web::Data<DbPool>,
    news_updater: web::Data<NewsUpdater>,
    query: web::Query<ByIdQuery>,
    comic_id: web::Path<i16>,
) -> Result<HttpResponse> {
    struct ComicId {
        id: i16,
    }

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
        *comic_id
    )
    .fetch_optional(&***conn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    let previous: Option<i16> = sqlx::query_as!(
        ComicId,
        r#"
			SELECT id
			FROM `comic`
			WHERE id < ?
				AND (? is NULL OR `isGuestComic` = ?)
				AND (? is NULL OR `isNonCanon` = ?)
			ORDER BY id DESC
		"#,
        *comic_id,
        is_guest_comic,
        is_guest_comic,
        is_non_canon,
        is_non_canon,
    )
    .fetch_optional(&***conn)
    .map_ok(|c| c.map(|i| i.id))
    .await
    .map_err(error::ErrorInternalServerError)?;

    let next: Option<i16> = sqlx::query_as!(
        ComicId,
        r#"
			SELECT id
			FROM `comic`
			WHERE id > ?
				AND (? is NULL OR `isGuestComic` = ?)
				AND (? is NULL OR `isNonCanon` = ?)
			ORDER BY id ASC
		"#,
        *comic_id,
        is_guest_comic,
        is_guest_comic,
        is_non_canon,
        is_non_canon,
    )
    .fetch_optional(&***conn)
    .map_ok(|c| c.map(|i| i.id))
    .await
    .map_err(error::ErrorInternalServerError)?;

    let news: Option<DatabaseNews> = if comic.is_some() {
        news_updater.check_for(*comic_id).await;

        sqlx::query_as!(
            DatabaseNews,
            r#"
				SELECT * FROM `news`
				WHERE `comic` = ?
			"#,
            *comic_id
        )
        .fetch_optional(&***conn)
        .await
        .map_err(error::ErrorInternalServerError)?
    } else {
        None
    };

    let editor_data = if let Some(token) = query.token {
        if is_token_valid(&conn, token)
            .await
            .map_err(error::ErrorInternalServerError)?
        {
            Some(fetch_editor_data_for_comic(&conn, *comic_id).await?)
        } else {
            None
        }
    } else {
        None
    };

    let mut items: BTreeMap<i16, Item> = sqlx::query_as!(
        Item,
        r#"
            SELECT i.*
            FROM items i
            JOIN occurences o ON o.items_id = i.id
            WHERE o.comic_id = ?
        "#,
        *comic_id,
    )
    .fetch(&***conn)
    .map_ok(|i| (i.id, i))
    .try_collect()
    .await
    .map_err(error::ErrorInternalServerError)?;

    let (comic_navigation_items, all_navigation_items) = if items.is_empty()
        && !matches!(query.include, Some(Inclusion::All))
    {
        (vec![], vec![])
    } else if let Some(Inclusion::All) = query.include {
        let mut all_items: BTreeMap<i16, Item> = sqlx::query_as!(
            Item,
            r#"
				SELECT *
				FROM items
			"#,
        )
        .fetch(&***conn)
        .map_ok(|i| (i.id, i))
        .try_collect()
        .await
        .map_err(error::ErrorInternalServerError)?;

        let mut all_navigation_items =
            fetch_all_item_navigation_data(&conn, *comic_id, is_guest_comic, is_non_canon).await?;
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

        transfer_item_data_to_navigation_item(&mut navigation_items_in_comic, &mut all_items);
        transfer_item_data_to_navigation_item(&mut all_navigation_items, &mut all_items);

        (navigation_items_in_comic, all_navigation_items)
    } else {
        let mut navigation_items_in_comic =
            fetch_comic_item_navigation_data(&conn, *comic_id, is_guest_comic, is_non_canon)
                .await?;

        transfer_item_data_to_navigation_item(&mut navigation_items_in_comic, &mut items);

        (navigation_items_in_comic, vec![])
    };

    let comic = if let Some(comic) = comic {
        Comic {
            comic: *comic_id,
            has_data: true,
            image_type: comic.ImageType.into(),
            publish_date: comic.publishDate.map(|nd| DateTime::from_utc(nd, Utc)),
            is_accurate_publish_date: comic.isAccuratePublishDate != 0,
            title: comic.title,
            tagline: comic.tagline,
            is_guest_comic: comic.isGuestComic != 0,
            is_non_canon: comic.isNonCanon != 0,
            has_no_cast: comic.HasNoCast != 0,
            has_no_location: comic.HasNoLocation != 0,
            has_no_storyline: comic.HasNoStoryline != 0,
            has_no_title: comic.HasNoTitle != 0,
            has_no_tagline: comic.HasNoTagline != 0,
            news: news.map_or(String::new(), |n| n.news),
            previous,
            next,
            editor_data,
            items: comic_navigation_items,
            all_items: all_navigation_items,
        }
    } else {
        todo!()
    };

    Ok(HttpResponse::Ok().json(comic))
}

fn transfer_item_data_to_navigation_item(
    navigation_items: &mut Vec<crate::models::ItemNavigationData>,
    items: &mut BTreeMap<i16, Item>,
) {
    for navigation_item in navigation_items {
        let Item {
            id: _,
            shortName: short_name,
            name,
            r#type,
            Color_Red: color_red,
            Color_Green: color_green,
            Color_Blue: color_blue,
        } = items.remove(&navigation_item.id).unwrap();
        navigation_item.short_name = Some(short_name);
        navigation_item.name = Some(name);
        navigation_item.r#type = Some(r#type);

        let color = format!("#{:02x}{:02x}{:02x}", color_red, color_green, color_blue);
        navigation_item.color = Some(color);
    }
}

async fn fetch_comic_list(
    conn: &DbPool,
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
    .fetch(&**conn)
    .map_ok(From::from)
    .try_collect()
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(comics)
}