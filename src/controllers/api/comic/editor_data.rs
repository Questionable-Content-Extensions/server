use std::convert::TryInto;

use crate::database::DbPoolConnection;
use crate::models::{ComicId, EditorData, ItemType, MissingNavigationData, NavigationData};
use actix_web::{error, Result};

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

async fn fetch_navigation_data_for_tagline(
    conn: &mut DbPoolConnection,
    comic_id: ComicId,
) -> Result<NavigationData> {
    let (first, last) = sqlx::query_as!(
        FirstLast,
        r#"
    		SELECT
    			MIN(c.id) as first,
    			MAX(c.id) as last
    		FROM comic c
    		WHERE (c.tagline IS NULL or NULLIF(c.tagline, '') IS NULL)
    			AND NOT c.HasNoTagline
    			AND c.id > 3132
    	"#
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(error::ErrorInternalServerError)?
    .map_or((None, None), |fl| (fl.first, fl.last));

    let previous = sqlx::query_scalar!(
        r#"
			SELECT c.id
			FROM comic c
			WHERE (c.tagline IS NULL OR NULLIF(c.tagline, '') IS NULL)
				AND NOT c.HasNoTagline
				AND c.id < ?
				AND c.id > 3132
			ORDER BY c.id DESC
			LIMIT 1
		"#,
        comic_id.into_inner()
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    let next = sqlx::query_scalar!(
        r#"
			SELECT c.id
			FROM comic c
			WHERE (c.tagline IS NULL OR NULLIF(c.tagline, '') IS NULL)
				AND NOT c.HasNoTagline
				AND c.id > ?
				AND c.id > 3132
			ORDER BY c.id ASC
			LIMIT 1
		"#,
        comic_id.into_inner()
    )
    .fetch_optional(&mut *conn)
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

async fn fetch_navigation_data_for_title(
    conn: &mut DbPoolConnection,
    comic_id: ComicId,
) -> Result<NavigationData> {
    let (first, last) = sqlx::query_as!(
        FirstLast,
        r#"
			SELECT
				MIN(c.id) as first,
				MAX(c.id) as last
			FROM comic c
			WHERE (c.title IS NULL or NULLIF(c.title, '') IS NULL)
				AND NOT c.HasNoTitle
		"#
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(error::ErrorInternalServerError)?
    .map_or((None, None), |fl| (fl.first, fl.last));

    let previous = sqlx::query_scalar!(
        r#"
			SELECT c.id
			FROM comic c
			WHERE (c.title IS NULL OR NULLIF(c.title, '') IS NULL)
				AND NOT c.HasNoTitle
				AND c.id < ?
			ORDER BY c.id DESC
			LIMIT 1
		"#,
        comic_id.into_inner()
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    let next = sqlx::query_scalar!(
        r#"
			SELECT c.id
			FROM comic c
			WHERE (c.title IS NULL OR NULLIF(c.title, '') IS NULL)
				AND NOT c.HasNoTitle
				AND c.id > ?
			ORDER BY c.id ASC
			LIMIT 1
		"#,
        comic_id.into_inner()
    )
    .fetch_optional(&mut *conn)
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

async fn fetch_first_for_item(
    conn: &mut DbPoolConnection,
    item: ItemType,
) -> Result<Option<ComicId>> {
    let item = item.as_str();
    let first = sqlx::query_scalar!(
        r#"
			SELECT c.id
			FROM comic c
			WHERE c.id NOT IN
				(
					SELECT o.comic_id
					FROM occurences o
					LEFT JOIN items i ON o.items_id = i.id
					WHERE i.type = ?
					AND o.comic_id = c.id
					GROUP BY o.comic_id
				)
				AND (? <> 'cast' OR NOT c.HasNoCast)
				AND (? <> 'location' OR NOT c.HasNoLocation)
				AND (? <> 'storyline' OR NOT c.HasNoStoryline)
			ORDER BY c.id ASC
			LIMIT 1
		"#,
        item,
        item,
        item,
        item
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(first
        .map(TryInto::try_into)
        .transpose()
        .expect("database has valid comicIds"))
}

async fn fetch_previous_for_item(
    conn: &mut DbPoolConnection,
    item: ItemType,
    comic_id: ComicId,
) -> Result<Option<ComicId>> {
    let item = item.as_str();
    let previous = sqlx::query_scalar!(
        r#"
			SELECT c.id
			FROM comic c
			WHERE c.id NOT IN
				(
					SELECT o.comic_id
					FROM occurences o
					LEFT JOIN items i ON o.items_id = i.id
					WHERE i.type = ?
					AND o.comic_id = c.id
					GROUP BY o.comic_id
				)
				AND c.id < ?
				AND (? <> 'cast' OR NOT c.HasNoCast)
				AND (? <> 'location' OR NOT c.HasNoLocation)
				AND (? <> 'storyline' OR NOT c.HasNoStoryline)
			ORDER BY c.id DESC
			LIMIT 1
		"#,
        item,
        comic_id.into_inner(),
        item,
        item,
        item,
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(previous
        .map(TryInto::try_into)
        .transpose()
        .expect("database has valid comicIds"))
}

async fn fetch_next_for_item(
    conn: &mut DbPoolConnection,
    item: ItemType,
    comic_id: ComicId,
) -> Result<Option<ComicId>> {
    let item = item.as_str();
    let next = sqlx::query_scalar!(
        r#"
			SELECT c.id
			FROM comic c
			WHERE c.id NOT IN
				(
					SELECT o.comic_id
					FROM occurences o
					LEFT JOIN items i ON o.items_id = i.id
					WHERE i.type = ?
					AND o.comic_id = c.id
					GROUP BY o.comic_id
				)
				AND c.id > ?
				AND (? <> 'cast' OR NOT c.HasNoCast)
				AND (? <> 'location' OR NOT c.HasNoLocation)
				AND (? <> 'storyline' OR NOT c.HasNoStoryline)
			ORDER BY c.id ASC
			LIMIT 1
		"#,
        item,
        comic_id.into_inner(),
        item,
        item,
        item,
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(next
        .map(TryInto::try_into)
        .transpose()
        .expect("database has valid comicIds"))
}

async fn fetch_last_for_item(
    conn: &mut DbPoolConnection,
    item: ItemType,
) -> Result<Option<ComicId>> {
    let item = item.as_str();
    let last = sqlx::query_scalar!(
        r#"
			SELECT c.id
			FROM comic c
			WHERE c.id NOT IN
				(
					SELECT o.comic_id
					FROM occurences o
					LEFT JOIN items i ON o.items_id = i.id
					WHERE i.type = ?
					AND o.comic_id = c.id
					GROUP BY o.comic_id
				)
				AND (? <> 'cast' OR NOT c.HasNoCast)
				AND (? <> 'location' OR NOT c.HasNoLocation)
				AND (? <> 'storyline' OR NOT c.HasNoStoryline)
			ORDER BY c.id DESC
			LIMIT 1
		"#,
        item,
        item,
        item,
        item,
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(last
        .map(TryInto::try_into)
        .transpose()
        .expect("database has valid comicIds"))
}

#[derive(Debug, sqlx::FromRow)]
struct FirstLast {
    first: Option<i16>,
    last: Option<i16>,
}
