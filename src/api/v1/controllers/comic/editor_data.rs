use crate::api::v1::models::{EditorData, ItemType, MissingNavigationData, NavigationData};
use crate::models::ComicId;
use actix_web::{error, Result};
use database::models::Comic as DatabaseComic;
use database::DbPoolConnection;
use std::convert::TryInto;

#[tracing::instrument(skip(conn))]
pub async fn fetch_editor_data_for_comic(
    conn: &mut DbPoolConnection,
    comic_id: ComicId,
) -> Result<EditorData> {
    let cast = fetch_navigation_data_for_item(&mut *conn, comic_id, ItemType::Cast).await?;
    let location = fetch_navigation_data_for_item(&mut *conn, comic_id, ItemType::Location).await?;
    let storyline =
        fetch_navigation_data_for_item(&mut *conn, comic_id, ItemType::Storyline).await?;
    let title = fetch_navigation_data_for_title(&mut *conn, comic_id).await?;
    let tagline = fetch_navigation_data_for_tagline(&mut *conn, comic_id).await?;

    Ok(EditorData {
        missing: MissingNavigationData {
            cast,
            location,
            storyline,
            title,
            tagline,
        },
    })
}

#[tracing::instrument(skip(conn))]
async fn fetch_navigation_data_for_tagline(
    conn: &mut DbPoolConnection,
    comic_id: ComicId,
) -> Result<NavigationData> {
    let (first, last) = DatabaseComic::first_and_last_missing_tagline(&mut **conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let previous =
        DatabaseComic::previous_missing_tagline_by_id(&mut **conn, comic_id.into_inner())
            .await
            .map_err(error::ErrorInternalServerError)?;

    let next = DatabaseComic::next_missing_tagline_by_id(&mut **conn, comic_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(NavigationData {
        first: first
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds"),
        previous: previous
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds"),
        next: next
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds"),
        last: last
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds"),
    })
}

#[tracing::instrument(skip(conn))]
async fn fetch_navigation_data_for_title(
    conn: &mut DbPoolConnection,
    comic_id: ComicId,
) -> Result<NavigationData> {
    let (first, last) = DatabaseComic::first_and_last_missing_title(&mut **conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let previous = DatabaseComic::previous_missing_title_by_id(&mut **conn, comic_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?;

    let next = DatabaseComic::next_missing_title_by_id(&mut **conn, comic_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(NavigationData {
        first: first
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds"),
        previous: previous
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds"),
        next: next
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds"),
        last: last
            .map(TryInto::try_into)
            .transpose()
            .expect("database has valid comicIds"),
    })
}

#[tracing::instrument(skip(conn))]
async fn fetch_navigation_data_for_item(
    conn: &mut DbPoolConnection,
    comic_id: ComicId,
    item: ItemType,
) -> Result<NavigationData> {
    let first = fetch_first_for_item(&mut *conn, item).await?;
    let previous = fetch_previous_for_item(&mut *conn, item, comic_id).await?;
    let next = fetch_next_for_item(&mut *conn, item, comic_id).await?;
    let last = fetch_last_for_item(&mut *conn, item).await?;

    Ok(NavigationData {
        first,
        previous,
        next,
        last,
    })
}

#[tracing::instrument(skip(conn))]
async fn fetch_first_for_item(
    conn: &mut DbPoolConnection,
    item: ItemType,
) -> Result<Option<ComicId>> {
    let item = item.as_str();
    let first = DatabaseComic::first_missing_items_by_type(
        &mut **conn,
        item.try_into().map_err(error::ErrorInternalServerError)?,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(first
        .map(TryInto::try_into)
        .transpose()
        .expect("database has valid comicIds"))
}

#[tracing::instrument(skip(conn))]
async fn fetch_previous_for_item(
    conn: &mut DbPoolConnection,
    item: ItemType,
    comic_id: ComicId,
) -> Result<Option<ComicId>> {
    let item = item.as_str();
    let previous = DatabaseComic::previous_missing_items_by_id_and_type(
        &mut **conn,
        comic_id.into_inner(),
        item.try_into().map_err(error::ErrorInternalServerError)?,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(previous
        .map(TryInto::try_into)
        .transpose()
        .expect("database has valid comicIds"))
}

#[tracing::instrument(skip(conn))]
async fn fetch_next_for_item(
    conn: &mut DbPoolConnection,
    item: ItemType,
    comic_id: ComicId,
) -> Result<Option<ComicId>> {
    let item = item.as_str();
    let next = DatabaseComic::next_missing_items_by_id_and_type(
        &mut **conn,
        comic_id.into_inner(),
        item.try_into().map_err(error::ErrorInternalServerError)?,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(next
        .map(TryInto::try_into)
        .transpose()
        .expect("database has valid comicIds"))
}

#[tracing::instrument(skip(conn))]
async fn fetch_last_for_item(
    conn: &mut DbPoolConnection,
    item: ItemType,
) -> Result<Option<ComicId>> {
    let item = item.as_str();
    let last = DatabaseComic::last_missing_items_by_type(
        &mut **conn,
        item.try_into().map_err(error::ErrorInternalServerError)?,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(last
        .map(TryInto::try_into)
        .transpose()
        .expect("database has valid comicIds"))
}
