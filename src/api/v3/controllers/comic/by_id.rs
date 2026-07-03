use crate::api::v3::controllers::comic::editor_data::fetch_editor_data_for_comic;
use crate::api::v3::controllers::comic::navigation_data::{
    ItemNavigationDataSorting, fetch_all_item_navigation_data, fetch_comic_item_navigation_data,
};
use crate::api::v3::models::{
    Comic, ComicData, EditorData, Exclusion, ImageType, Inclusion, ItemNavigationData,
    MissingComic, MissingEditorData, PresentComic, Sorting,
};
use crate::models::{ComicId, False, Token, True};
use crate::util::NewsUpdater;
use actix_web::web::Json;
use actix_web::{Result, error, web};
use actix_web_grants::authorities::{AuthDetails, AuthoritiesCheck};
use api_macros::api_endpoint;
use chrono::{TimeZone, Utc};
use database::DbPool;
use database::models::{Comic as DatabaseComic, Item as DatabaseItem};
use serde::Deserialize;
use shared::token_permissions;
use std::convert::TryInto;
use tracing::{Instrument, info_span};
use ts_rs::TS;

#[api_endpoint(method = "GET", path = "comicdata/{comicId}")]
#[tracing::instrument(skip(pool, news_updater, auth), fields(permissions = ?auth.authorities))]
#[expect(clippy::too_many_lines)]
pub async fn by_id(
    pool: web::Data<DbPool>,
    news_updater: web::Data<NewsUpdater>,
    query: web::Query<ByIdQuery>,
    comic_id: web::Path<ComicId>,
    auth: AuthDetails,
) -> Result<Json<Comic>> {
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

    let include_hidden = auth.has_authority(token_permissions::HAS_VALID_TOKEN);
    let comic = DatabaseComic::by_id_with_navigation_and_news(
        &mut *conn,
        comic_id.into_inner(),
        include_guest_comics,
        include_non_canon_comics,
        include_hidden,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    if comic.is_some() {
        news_updater.check_for(comic_id);
    }

    let editor_data = if include_hidden {
        fetch_editor_data_for_comic(&mut conn, comic_id).await?
    } else {
        EditorData::Missing(MissingEditorData::default())
    };

    let (comic_navigation_items, all_navigation_items) =
        if matches!(query.include, Some(Inclusion::All)) {
            let mut all_navigation_items: Vec<ItemNavigationData> = fetch_all_item_navigation_data(
                &mut conn,
                comic_id,
                include_guest_comics,
                include_non_canon_comics,
                match query.sorting {
                    Some(Sorting::ByLastAppearance) => ItemNavigationDataSorting::ByLastAppearance,
                    Some(Sorting::ByCount) | None => ItemNavigationDataSorting::ByCount,
                },
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
                image_type: Some(ImageType::from_trusted(comic.image_type)),
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
                news: comic.news,
                previous: comic
                    .prev_id
                    .map(TryInto::try_into)
                    .transpose()
                    .expect("database has valid comicIds"),
                next: comic
                    .next_id
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

    Ok(Json(comic))
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct ByIdQuery {
    // This is never read because it's used by the auth middleware only.
    // We still include it here so it ends up in the TS binding.
    #[expect(dead_code)]
    #[ts(optional)]
    pub token: Option<Token>,

    #[ts(optional)]
    exclude: Option<Exclusion>,
    #[ts(optional)]
    include: Option<Inclusion>,
    #[ts(optional)]
    sorting: Option<Sorting>,
}
