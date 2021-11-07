use crate::models::{ComicId, ItemNavigationData, NavigationData};
use actix_web::{error, Result};
use database::models::Item as DatabaseItem;
use database::DbPoolConnection;
use std::convert::TryInto;

#[allow(clippy::too_many_lines)]
pub async fn fetch_all_item_navigation_data(
    conn: &mut DbPoolConnection,
    comic_id: ComicId,
    include_guest_comics: Option<bool>,
    include_non_canon_comics: Option<bool>,
) -> Result<Vec<ItemNavigationData>> {
    let first_last_counts = DatabaseItem::first_and_last_apperances_and_count(
        &mut *conn,
        include_guest_comics,
        include_non_canon_comics,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    let previous = DatabaseItem::previous_apperances_by_comic_id_mapped_by_id(
        &mut *conn,
        comic_id.into_inner(),
        include_guest_comics,
        include_non_canon_comics,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    let next = DatabaseItem::next_apperances_by_comic_id_mapped_by_id(
        &mut *conn,
        comic_id.into_inner(),
        include_guest_comics,
        include_non_canon_comics,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(first_last_counts
        .into_iter()
        .map(|flc| ItemNavigationData {
            id: flc.id.into(),
            navigation_data: NavigationData {
                first: flc
                    .first
                    .map(TryInto::try_into)
                    .transpose()
                    .expect("database has valid comicIds"),
                previous: previous
                    .get(&flc.id)
                    .copied()
                    .map(TryInto::try_into)
                    .transpose()
                    .expect("database has valid comicIds"),
                next: next
                    .get(&flc.id)
                    .copied()
                    .map(TryInto::try_into)
                    .transpose()
                    .expect("database has valid comicIds"),
                last: flc
                    .last
                    .map(TryInto::try_into)
                    .transpose()
                    .expect("database has valid comicIds"),
            },
            count: flc.count,
            short_name: None,
            name: None,
            r#type: None,
            color: None,
        })
        .collect())
}

#[allow(clippy::too_many_lines)]
pub async fn fetch_comic_item_navigation_data(
    conn: &mut DbPoolConnection,
    comic_id: ComicId,
    include_guest_comics: Option<bool>,
    include_non_canon_comics: Option<bool>,
) -> Result<Vec<ItemNavigationData>> {
    let first_last_counts =
        DatabaseItem::first_and_last_apperances_and_count_of_items_in_comic_by_comic_id(
            &mut *conn,
            comic_id.into_inner(),
            include_guest_comics,
            include_non_canon_comics,
        )
        .await
        .map_err(error::ErrorInternalServerError)?;

    let previous = DatabaseItem::previous_apperances_of_items_in_comic_by_comic_id(
        &mut *conn,
        comic_id.into_inner(),
        include_guest_comics,
        include_non_canon_comics,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    let next = DatabaseItem::next_apperances_of_items_in_comic_by_comic_id(
        &mut *conn,
        comic_id.into_inner(),
        include_guest_comics,
        include_non_canon_comics,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(first_last_counts
        .into_iter()
        .map(|flc| ItemNavigationData {
            id: flc.id.into(),
            navigation_data: NavigationData {
                first: flc
                    .first
                    .map(TryInto::try_into)
                    .transpose()
                    .expect("database has valid comicIds"),
                previous: previous
                    .get(&flc.id)
                    .copied()
                    .map(TryInto::try_into)
                    .transpose()
                    .expect("database has valid comicIds"),
                next: next
                    .get(&flc.id)
                    .copied()
                    .map(TryInto::try_into)
                    .transpose()
                    .expect("database has valid comicIds"),
                last: flc
                    .last
                    .map(TryInto::try_into)
                    .transpose()
                    .expect("database has valid comicIds"),
            },
            count: flc.count,
            short_name: None,
            name: None,
            r#type: None,
            color: None,
        })
        .collect())
}
