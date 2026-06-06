use crate::api::v3::models::{
    EditorData, MissingNavigationData, NavigationData, PresentEditorData,
};
use crate::models::{ComicId, True};
use actix_web::{Result, error};
use database::DbPoolConnection;
use database::models::{Comic as DatabaseComic, NavigationResult};
use std::convert::TryInto;

#[tracing::instrument(skip(conn))]
pub async fn fetch_editor_data_for_comic(
    conn: &mut DbPoolConnection,
    comic_id: ComicId,
) -> Result<EditorData> {
    let id = comic_id.into_inner();

    let cast_nav = DatabaseComic::missing_cast_navigation(&mut **conn, id)
        .await
        .map_err(error::ErrorInternalServerError)?;
    let location_nav = DatabaseComic::missing_location_navigation(&mut **conn, id)
        .await
        .map_err(error::ErrorInternalServerError)?;
    let storyline_nav = DatabaseComic::missing_storyline_navigation(&mut **conn, id)
        .await
        .map_err(error::ErrorInternalServerError)?;
    let title_nav = DatabaseComic::missing_title_navigation(&mut **conn, id)
        .await
        .map_err(error::ErrorInternalServerError)?;
    let tagline_nav = DatabaseComic::missing_tagline_navigation(&mut **conn, id)
        .await
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
