use crate::controllers::api::comic::editor_data::fetch_editor_data_for_comic;
use crate::controllers::api::comic::navigation_data::{
    fetch_all_item_navigation_data, fetch_comic_item_navigation_data,
};
use crate::models::{
    Comic, ComicData, ComicId, EditorData, Exclusion, False, Inclusion, ItemColor, ItemType,
    MissingComic, MissingEditorData, PresentComic, True,
};
use crate::util::NewsUpdater;
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::{AuthDetails, PermissionsCheck};
use chrono::{TimeZone, Utc};
use database::models::{Comic as DatabaseComic, Item as DatabaseItem, News as DatabaseNews};
use database::DbPool;
use serde::Deserialize;
use shared::token_permissions;
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};

#[allow(clippy::too_many_lines)]
pub(crate) async fn by_id(
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
        news_updater.check_for(comic_id);

        DatabaseNews::by_comic_id(&mut conn, comic_id.into_inner())
            .await
            .map_err(error::ErrorInternalServerError)?
    } else {
        None
    };

    let editor_data = if auth.has_permission(token_permissions::HAS_VALID_TOKEN) {
        EditorData::Present(fetch_editor_data_for_comic(&mut conn, comic_id).await?)
    } else {
        EditorData::Missing(MissingEditorData::default())
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
            editor_data,
            all_items: all_navigation_items,
            data: ComicData::Present(PresentComic {
                has_data: True::default(),
                image_type: Some(comic.image_type.into()),
                publish_date: comic.publish_date.map(|nd| Utc.from_utc_datetime(&nd)),
                is_accurate_publish_date: comic.is_accurate_publish_date != 0,
                title: Some(comic.title),
                tagline: comic.tagline,
                is_guest_comic: comic.is_guest_comic != 0,
                is_non_canon: comic.is_non_canon != 0,
                has_no_cast: comic.has_no_cast != 0,
                has_no_location: comic.has_no_location != 0,
                has_no_storyline: comic.has_no_storyline != 0,
                has_no_title: comic.has_no_title != 0,
                has_no_tagline: comic.has_no_tagline != 0,
                news: news.map(|n| n.news),
                previous: previous
                    .map(TryInto::try_into)
                    .transpose()
                    .expect("database has valid comicIds"),
                next: next
                    .map(TryInto::try_into)
                    .transpose()
                    .expect("database has valid comicIds"),
                items: comic_navigation_items,
            }),
        }
    } else {
        Comic {
            comic: comic_id,
            editor_data,
            all_items: all_navigation_items,
            data: ComicData::Missing(MissingComic {
                has_data: False::default(),
            }),
        }
    };

    Ok(HttpResponse::Ok().json(comic))
}

fn transfer_item_data_to_navigation_item(
    navigation_items: &mut Vec<crate::models::ItemNavigationData>,
    items: &mut BTreeMap<u16, DatabaseItem>,
) -> anyhow::Result<()> {
    for navigation_item in navigation_items {
        let DatabaseItem {
            id: _,
            short_name,
            name,
            r#type,
            color_red,
            color_green,
            color_blue,
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

#[derive(Debug, Deserialize)]
pub(crate) struct ByIdQuery {
    exclude: Option<Exclusion>,
    include: Option<Inclusion>,
}
