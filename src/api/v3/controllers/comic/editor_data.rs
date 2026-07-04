use crate::api::v3::models::{
    EditorData, MissingNavigationData, NavigationData, PresentEditorData,
};
use crate::models::{ComicId, True};
use actix_web::{Result, error};
use database::DbPool;
use database::models::{Comic as DatabaseComic, NavigationResult};
use std::convert::TryInto;

#[tracing::instrument(skip(pool))]
pub async fn fetch_editor_data_for_comic(pool: &DbPool, comic_id: ComicId) -> Result<EditorData> {
    let id = comic_id.into_inner();

    let mut cast_conn = pool
        .acquire()
        .await
        .map_err(error::ErrorInternalServerError)?;
    let mut location_conn = pool
        .acquire()
        .await
        .map_err(error::ErrorInternalServerError)?;
    let mut storyline_conn = pool
        .acquire()
        .await
        .map_err(error::ErrorInternalServerError)?;
    let mut title_conn = pool
        .acquire()
        .await
        .map_err(error::ErrorInternalServerError)?;
    let mut tagline_conn = pool
        .acquire()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let (cast_nav, location_nav, storyline_nav, title_nav, tagline_nav) = tokio::try_join!(
        DatabaseComic::missing_cast_navigation(&mut *cast_conn, id),
        DatabaseComic::missing_location_navigation(&mut *location_conn, id),
        DatabaseComic::missing_storyline_navigation(&mut *storyline_conn, id),
        DatabaseComic::missing_title_navigation(&mut *title_conn, id),
        DatabaseComic::missing_tagline_navigation(&mut *tagline_conn, id),
    )
    .map_err(error::ErrorInternalServerError)?;

    Ok(EditorData::Present(PresentEditorData {
        present: True::default(),
        missing: MissingNavigationData {
            cast: nav_result_to_navigation_data(&cast_nav),
            location: nav_result_to_navigation_data(&location_nav),
            storyline: nav_result_to_navigation_data(&storyline_nav),
            title: nav_result_to_navigation_data(&title_nav),
            tagline: nav_result_to_navigation_data(&tagline_nav),
        },
    }))
}

fn nav_result_to_navigation_data(nav: &NavigationResult) -> NavigationData {
    NavigationData {
        first: nav
            .first
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds"),
        previous: nav
            .previous
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds"),
        next: nav
            .next
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds"),
        last: nav
            .last
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds"),
    }
}
