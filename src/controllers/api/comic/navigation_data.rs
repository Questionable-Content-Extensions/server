use std::collections::BTreeMap;
use std::convert::TryInto;

use crate::database::DbPoolConnection;
use crate::models::{ComicId, ItemNavigationData, NavigationData};
use actix_web::{error, Result};
use futures::TryStreamExt;

#[allow(clippy::too_many_lines)]
pub async fn fetch_all_item_navigation_data(
    conn: &mut DbPoolConnection,
    comic_id: ComicId,
    is_guest_comic: Option<bool>,
    is_non_canon: Option<bool>,
) -> Result<Vec<ItemNavigationData>> {
    let first_last_counts = sqlx::query_as!(
        FirstLastCount,
        r#"
			SELECT
				i.id,
				MIN(c.id) as first,
				MAX(c.id) as last,
				COUNT(c.id) as count
			FROM items i
			JOIN occurences o ON o.items_id = i.id
			JOIN comic c ON c.id = o.comic_id
				AND (? is NULL OR c.isGuestComic = ?)
				AND (? is NULL OR c.isNonCanon = ?)
			GROUP by i.id
			ORDER BY count DESC
		"#,
        is_guest_comic,
        is_guest_comic,
        is_non_canon,
        is_non_canon,
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    let previous: BTreeMap<i16, i16> = sqlx::query_as!(
        PrevNext,
        r#"
			SELECT i.id as id, MAX(c.id) as comic
			FROM items i
			JOIN occurences o ON o.items_id = i.id
			JOIN comic c ON c.id = o.comic_id
			WHERE c.id < ?
				AND (? is NULL OR c.isGuestComic = ?)
				AND (? is NULL OR c.isNonCanon = ?)
			GROUP BY i.id
		"#,
        comic_id.into_inner(),
        is_guest_comic,
        is_guest_comic,
        is_non_canon,
        is_non_canon,
    )
    .fetch(&mut *conn)
    .try_filter_map(|pn| async move {
        if let Some(comic) = pn.comic {
            Ok(Some((pn.id, comic)))
        } else {
            Ok(None)
        }
    })
    .try_collect()
    .await
    .map_err(error::ErrorInternalServerError)?;

    let next: BTreeMap<i16, i16> = sqlx::query_as!(
        PrevNext,
        r#"
			SELECT i.id as id, MIN(c.id) as comic
			FROM items i
			JOIN occurences o ON o.items_id = i.id
			JOIN comic c ON c.id = o.comic_id
			WHERE c.id > ?
				AND (? is NULL OR c.isGuestComic = ?)
				AND (? is NULL OR c.isNonCanon = ?)
			GROUP BY i.id
		"#,
        comic_id.into_inner(),
        is_guest_comic,
        is_guest_comic,
        is_non_canon,
        is_non_canon,
    )
    .fetch(&mut *conn)
    .try_filter_map(|pn| async move {
        if let Some(comic) = pn.comic {
            Ok(Some((pn.id, comic)))
        } else {
            Ok(None)
        }
    })
    .try_collect()
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
    is_guest_comic: Option<bool>,
    is_non_canon: Option<bool>,
) -> Result<Vec<ItemNavigationData>> {
    let first_last_counts = sqlx::query_as!(
        FirstLastCount,
        r#"
			SELECT
				i.id,
				MIN(c.id) as first,
				MAX(c.id) as last,
				COUNT(c.id) as count
			FROM items i
			JOIN occurences o ON o.items_id = i.id
			JOIN comic c ON c.id = o.comic_id
				AND (? is NULL OR c.isGuestComic = ?)
				AND (? is NULL OR c.isNonCanon = ?)
			WHERE i.id IN (
				SELECT items_id FROM `occurences` WHERE comic_id = ?
			)
			GROUP by i.id
			ORDER BY count DESC
		"#,
        is_guest_comic,
        is_guest_comic,
        is_non_canon,
        is_non_canon,
        comic_id.into_inner()
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    let previous: BTreeMap<i16, i16> = sqlx::query_as!(
        PrevNext,
        r#"
			SELECT i.id as id, MAX(c.id) as comic
			FROM items i
			JOIN occurences o ON o.items_id = i.id
			JOIN comic c ON c.id = o.comic_id
			WHERE c.id < ?
				AND i.id IN (
					SELECT items_id FROM `occurences` WHERE comic_id = ?
				)
				AND (? is NULL OR c.isGuestComic = ?)
				AND (? is NULL OR c.isNonCanon = ?)
			GROUP BY i.id
		"#,
        comic_id.into_inner(),
        comic_id.into_inner(),
        is_guest_comic,
        is_guest_comic,
        is_non_canon,
        is_non_canon,
    )
    .fetch(&mut *conn)
    .try_filter_map(|pn| async move {
        if let Some(comic) = pn.comic {
            Ok(Some((pn.id, comic)))
        } else {
            Ok(None)
        }
    })
    .try_collect()
    .await
    .map_err(error::ErrorInternalServerError)?;

    let next: BTreeMap<i16, i16> = sqlx::query_as!(
        PrevNext,
        r#"
			SELECT i.id as id, MIN(c.id) as comic
			FROM items i
			JOIN occurences o ON o.items_id = i.id
			JOIN comic c ON c.id = o.comic_id
			WHERE c.id > ?
			AND i.id IN (
				SELECT items_id FROM `occurences` WHERE comic_id = ?
			)
				AND (? is NULL OR c.isGuestComic = ?)
				AND (? is NULL OR c.isNonCanon = ?)
			GROUP BY i.id
		"#,
        comic_id.into_inner(),
        comic_id.into_inner(),
        is_guest_comic,
        is_guest_comic,
        is_non_canon,
        is_non_canon,
    )
    .fetch(&mut *conn)
    .try_filter_map(|pn| async move {
        if let Some(comic) = pn.comic {
            Ok(Some((pn.id, comic)))
        } else {
            Ok(None)
        }
    })
    .try_collect()
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

#[derive(Debug, sqlx::FromRow)]
struct FirstLastCount {
    id: i16,
    first: Option<i16>,
    last: Option<i16>,
    count: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct PrevNext {
    id: i16,
    comic: Option<i16>,
}
