use crate::api::v2::models::{NavigationData, UnhydratedItemNavigationData};
use crate::models::ComicId;
use actix_web::{Result, error};
use database::DbPoolConnection;
use database::models::{
    Item as DatabaseItem, ItemFirstLastCount, ItemId as DatabaseItemId, PreviousAppearances,
};
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;

#[derive(Clone, Copy, Debug)]
pub enum ItemNavigationDataSorting {
    ByCount,
    ByLastAppearance,
}

#[tracing::instrument(skip(conn))]
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
        ItemNavigationDataSorting::ByLastAppearance => Ok(order_by_last_appearance(
            &first_last_counts,
            &last_appearance_order,
        )
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
    }
}

/// Orders `first_last_counts` by last appearance: items in `last_appearance_order` come first
/// (preserving that order), followed by any remaining items not in `last_appearance_order`.
/// Builds O(1) lookup structures so the overall operation is O(n) instead of O(n²).
fn order_by_last_appearance<'a>(
    first_last_counts: &'a [ItemFirstLastCount],
    last_appearance_order: &[DatabaseItemId],
) -> Vec<&'a ItemFirstLastCount> {
    let flc_map: HashMap<u16, &ItemFirstLastCount> =
        first_last_counts.iter().map(|flc| (flc.id, flc)).collect();
    let seen: HashSet<u16> = last_appearance_order
        .iter()
        .map(|i| *i.as_inner())
        .collect();

    last_appearance_order
        .iter()
        .map(DatabaseItemId::as_inner)
        .chain(
            first_last_counts
                .iter()
                .filter(move |flc| !seen.contains(&flc.id))
                .map(|flc| &flc.id),
        )
        .map(|id| *flc_map.get(id).expect("database has valid itemIds"))
        .collect()
}

#[tracing::instrument(skip(conn))]
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

#[cfg(test)]
mod tests {
    use super::*;
    use database::models::ItemFirstLastCount;

    fn flc(id: u16) -> ItemFirstLastCount {
        ItemFirstLastCount {
            id,
            first: None,
            last: None,
            count: 0,
        }
    }

    fn order(ids: &[u16]) -> Vec<DatabaseItemId> {
        ids.iter().copied().map(DatabaseItemId::from).collect()
    }

    #[test]
    fn items_in_order_come_first_then_rest() {
        let counts = vec![flc(1), flc(2), flc(3), flc(4)];
        let appearance_order = order(&[3, 1]);

        let result: Vec<u16> = order_by_last_appearance(&counts, &appearance_order)
            .into_iter()
            .map(|flc| flc.id)
            .collect();

        assert_eq!(&result[..2], &[3, 1]);
        let tail: std::collections::HashSet<u16> = result[2..].iter().copied().collect();
        assert_eq!(tail, std::collections::HashSet::from([2, 4]));
    }

    #[test]
    fn empty_appearance_order_returns_all_items() {
        let counts = vec![flc(10), flc(20)];
        let result = order_by_last_appearance(&counts, &[]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn all_items_in_appearance_order_no_tail() {
        let counts = vec![flc(5), flc(6), flc(7)];
        let appearance_order = order(&[7, 5, 6]);

        let result: Vec<u16> = order_by_last_appearance(&counts, &appearance_order)
            .into_iter()
            .map(|flc| flc.id)
            .collect();

        assert_eq!(result, vec![7, 5, 6]);
    }
}
