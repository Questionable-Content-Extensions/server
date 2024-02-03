use crate::api::v2::models::{NavigationData, UnhydratedItemNavigationData};
use crate::models::ComicId;
use actix_web::{error, Result};
use database::models::{Item as DatabaseItem, ItemId as DatabaseItemId, PreviousAppearances};
use database::DbPoolConnection;
use std::convert::TryInto;

#[derive(Clone, Copy, Debug)]
pub enum ItemNavigationDataSorting {
    ByCount,
    ByLastAppearance,
}

#[tracing::instrument(skip(conn))]
#[allow(clippy::too_many_lines)]
pub async fn fetch_all_item_navigation_data(
    conn: &mut DbPoolConnection,
    comic_id: ComicId,
    include_guest_comics: Option<bool>,
    include_non_canon_comics: Option<bool>,
    sorting: ItemNavigationDataSorting,
) -> Result<Vec<UnhydratedItemNavigationData>> {
    let first_last_counts = DatabaseItem::first_and_last_apperances_and_count(
        &mut **conn,
        include_guest_comics,
        include_non_canon_comics,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    let PreviousAppearances {
        appearances: previous,
        order: last_appearance_order,
    } = DatabaseItem::previous_apperances_by_comic_id_mapped_by_id(
        &mut **conn,
        comic_id.into_inner(),
        include_guest_comics,
        include_non_canon_comics,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    let next = DatabaseItem::next_apperances_by_comic_id_mapped_by_id(
        &mut **conn,
        comic_id.into_inner(),
        include_guest_comics,
        include_non_canon_comics,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    match sorting {
        ItemNavigationDataSorting::ByCount => Ok(first_last_counts
            .into_iter()
            .map(|flc| UnhydratedItemNavigationData {
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
            })
            .collect()),
        ItemNavigationDataSorting::ByLastAppearance => Ok(last_appearance_order
            .iter()
            .map(DatabaseItemId::as_inner)
            .chain(
                first_last_counts
                    .iter()
                    .filter(|flc| !last_appearance_order.iter().any(|&i| i == flc.id))
                    .map(|flc| &flc.id),
            )
            .map(|&id| {
                first_last_counts
                    .iter()
                    .find(|flc| flc.id == id)
                    .expect("database has valid itemIds")
            })
            .map(|flc| UnhydratedItemNavigationData {
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
            })
            .collect()),
    }
}

#[tracing::instrument(skip(conn))]
#[allow(clippy::too_many_lines)]
pub async fn fetch_comic_item_navigation_data(
    conn: &mut DbPoolConnection,
    comic_id: ComicId,
    include_guest_comics: Option<bool>,
    include_non_canon_comics: Option<bool>,
) -> Result<Vec<UnhydratedItemNavigationData>> {
    let first_last_counts =
        DatabaseItem::first_and_last_apperances_and_count_of_items_in_comic_by_comic_id(
            &mut **conn,
            comic_id.into_inner(),
            include_guest_comics,
            include_non_canon_comics,
        )
        .await
        .map_err(error::ErrorInternalServerError)?;

    let previous = DatabaseItem::previous_apperances_of_items_in_comic_by_comic_id(
        &mut **conn,
        comic_id.into_inner(),
        include_guest_comics,
        include_non_canon_comics,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    let next = DatabaseItem::next_apperances_of_items_in_comic_by_comic_id(
        &mut **conn,
        comic_id.into_inner(),
        include_guest_comics,
        include_non_canon_comics,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(first_last_counts
        .into_iter()
        .map(|flc| UnhydratedItemNavigationData {
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
        })
        .collect())
}
