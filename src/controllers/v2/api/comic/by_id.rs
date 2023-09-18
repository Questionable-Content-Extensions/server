use crate::controllers::v1::api::comic::editor_data::fetch_editor_data_for_comic;
use crate::controllers::v1::api::comic::navigation_data::{
    fetch_all_item_navigation_data, fetch_comic_item_navigation_data,
};
use crate::models::v2::{
    Comic, ComicData, ComicId, EditorData, Exclusion, False, Inclusion, ItemNavigationData,
    MissingComic, MissingEditorData, PresentComic, Token, True,
};
use crate::util::NewsUpdater;
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::{AuthDetails, PermissionsCheck};
use chrono::{TimeZone, Utc};
use database::models::{Comic as DatabaseComic, Item as DatabaseItem, News as DatabaseNews};
use database::DbPool;
use serde::Deserialize;
use shared::token_permissions;
use std::convert::TryInto;
use tracing::{info_span, Instrument};
use ts_rs::TS;

#[tracing::instrument(skip(pool, news_updater, auth), fields(permissions = ?auth.permissions))]
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
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let (include_guest_comics, include_non_canon_comics) = match query.exclude {
        None => (None, None),
        Some(Exclusion::Guest) => (Some(false), None),
        Some(Exclusion::NonCanon) => (None, Some(false)),
    };

    let comic = DatabaseComic::by_id(&mut *conn, comic_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?;

    let previous = DatabaseComic::previous_id(
        &mut *conn,
        comic_id.into_inner(),
        include_guest_comics,
        include_non_canon_comics,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    let next = DatabaseComic::next_id(
        &mut *conn,
        comic_id.into_inner(),
        include_guest_comics,
        include_non_canon_comics,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    let news: Option<DatabaseNews> = if comic.is_some() {
        news_updater.check_for(comic_id);

        DatabaseNews::by_comic_id(&mut *conn, comic_id.into_inner())
            .await
            .map_err(error::ErrorInternalServerError)?
    } else {
        None
    };

    let editor_data = if auth.has_permission(token_permissions::HAS_VALID_TOKEN) {
        EditorData::Present(
            fetch_editor_data_for_comic(&mut conn, comic_id)
                .await?
                .into(),
        )
    } else {
        EditorData::Missing(MissingEditorData::default())
    };

    let (comic_navigation_items, all_navigation_items) = if let Some(Inclusion::All) = query.include
    {
        let mut all_navigation_items: Vec<ItemNavigationData> = fetch_all_item_navigation_data(
            &mut conn,
            comic_id,
            include_guest_comics,
            include_non_canon_comics,
        )
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

        let item_ids_in_comic =
            DatabaseItem::occurrences_in_comic_by_id(&mut *conn, comic_id.into_inner())
                .await
                .map_err(error::ErrorInternalServerError)?;

        let mut comic_navigation_items = Vec::with_capacity(item_ids_in_comic.len());
        let mut i = 0;
        while i < all_navigation_items.len() {
            let navigation_item = &mut all_navigation_items[i];
            if item_ids_in_comic.contains(&navigation_item.id.into_inner()) {
                let element = all_navigation_items.remove(i);
                comic_navigation_items.push(element);
            } else {
                i += 1;
            }
        }
        (comic_navigation_items, all_navigation_items)
    } else {
        let comic_navigation_items = fetch_comic_item_navigation_data(
            &mut conn,
            comic_id,
            include_guest_comics,
            include_non_canon_comics,
        )
        .await?;

        (
            comic_navigation_items.into_iter().map(Into::into).collect(),
            Vec::new(),
        )
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
                title: comic.title,
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

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub(crate) struct ByIdQuery {
    // This is never read because it's used by the auth middleware only.
    // We still include it here so it ends up in the TS binding.
    #[allow(dead_code)]
    #[ts(optional)]
    pub token: Option<Token>,

    #[ts(optional)]
    exclude: Option<Exclusion>,
    #[ts(optional)]
    include: Option<Inclusion>,
}
