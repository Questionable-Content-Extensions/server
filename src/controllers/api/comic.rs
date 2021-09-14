use crate::controllers::api::comic::editor_data::fetch_editor_data_for_comic;
use crate::controllers::api::comic::navigation_data::{
    fetch_all_item_navigation_data, fetch_comic_item_navigation_data,
};
use crate::database::models::{Comic as DatabaseComic, Item as DatabaseItem, News as DatabaseNews};
use crate::database::DbPool;
use crate::models::{
    token_permissions, Comic, ComicList, Exclusion, Inclusion, ItemColor, ItemType,
};
use crate::util::{log_action, NewsUpdater};
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::{AuthDetails, PermissionsCheck};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use futures::{TryFutureExt, TryStreamExt};
use log::info;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::convert::TryFrom;

mod editor_data;
pub(crate) mod navigation_data;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(all)))
        .service(web::resource("/excluded").route(web::get().to(excluded)))
        .service(web::resource("/additem").route(web::post().to(add_item)))
        .service(web::resource("/removeitem").route(web::post().to(remove_item)))
        .service(web::resource("/settitle").route(web::post().to(set_title)))
        .service(web::resource("/{comicId}").route(web::get().to(by_id)));
}

async fn all(pool: web::Data<DbPool>, query: web::Query<AllQuery>) -> Result<HttpResponse> {
    let (is_guest_comic, is_non_canon) = match query.exclude {
        None => (None, None),
        Some(Exclusion::Guest) => (Some(false), None),
        Some(Exclusion::NonCanon) => (None, Some(false)),
    };

    info!(
        "Requesting all comics (exclude guest comics: {}, exclude non-canon comics: {})",
        is_guest_comic.map_or(false, |v| !v),
        is_non_canon.map_or(false, |v| !v)
    );

    Ok(HttpResponse::Ok().json(fetch_comic_list(&pool, is_guest_comic, is_non_canon).await?))
}

async fn excluded(pool: web::Data<DbPool>, query: web::Query<AllQuery>) -> Result<HttpResponse> {
    let (is_guest_comic, is_non_canon) = match query.exclude {
        None => {
            return Err(error::ErrorBadRequest(
                "exclude parameter must be set to either `guest` or `non-canon`",
            ))
        }
        Some(Exclusion::Guest) => (Some(true), None),
        Some(Exclusion::NonCanon) => (None, Some(true)),
    };

    info!(
        "Requesting excluded {} comics",
        if is_guest_comic.is_some() {
            "guest"
        } else {
            "non-canon"
        }
    );

    Ok(HttpResponse::Ok().json(fetch_comic_list(&pool, is_guest_comic, is_non_canon).await?))
}

#[allow(clippy::too_many_lines)]
async fn by_id(
    pool: web::Data<DbPool>,
    news_updater: web::Data<NewsUpdater>,
    query: web::Query<ByIdQuery>,
    comic_id: web::Path<i16>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    struct ComicId {
        id: i16,
    }

    let mut conn = pool
        .acquire()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let (is_guest_comic, is_non_canon) = match query.exclude {
        None => (None, None),
        Some(Exclusion::Guest) => (Some(false), None),
        Some(Exclusion::NonCanon) => (None, Some(false)),
    };

    let comic: Option<DatabaseComic> = sqlx::query_as!(
        DatabaseComic,
        r#"
		SELECT * FROM `comic`
		WHERE `id` = ?
	"#,
        *comic_id
    )
    .fetch_optional(&mut conn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    let previous: Option<i16> = sqlx::query_as!(
        ComicId,
        r#"
			SELECT id
			FROM `comic`
			WHERE id < ?
				AND (? is NULL OR `isGuestComic` = ?)
				AND (? is NULL OR `isNonCanon` = ?)
			ORDER BY id DESC
		"#,
        *comic_id,
        is_guest_comic,
        is_guest_comic,
        is_non_canon,
        is_non_canon,
    )
    .fetch_optional(&mut conn)
    .map_ok(|c| c.map(|i| i.id))
    .await
    .map_err(error::ErrorInternalServerError)?;

    let next: Option<i16> = sqlx::query_as!(
        ComicId,
        r#"
			SELECT id
			FROM `comic`
			WHERE id > ?
				AND (? is NULL OR `isGuestComic` = ?)
				AND (? is NULL OR `isNonCanon` = ?)
			ORDER BY id ASC
		"#,
        *comic_id,
        is_guest_comic,
        is_guest_comic,
        is_non_canon,
        is_non_canon,
    )
    .fetch_optional(&mut conn)
    .map_ok(|c| c.map(|i| i.id))
    .await
    .map_err(error::ErrorInternalServerError)?;

    let news: Option<DatabaseNews> = if comic.is_some() {
        news_updater.check_for(*comic_id).await;

        sqlx::query_as!(
            DatabaseNews,
            r#"
				SELECT * FROM `news`
				WHERE `comic` = ?
			"#,
            *comic_id
        )
        .fetch_optional(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?
    } else {
        None
    };

    let editor_data = if auth.has_permission(token_permissions::HAS_VALID_TOKEN) {
        Some(fetch_editor_data_for_comic(&mut conn, *comic_id).await?)
    } else {
        None
    };

    let mut items: BTreeMap<i16, DatabaseItem> = sqlx::query_as!(
        DatabaseItem,
        r#"
            SELECT i.*
            FROM items i
            JOIN occurences o ON o.items_id = i.id
            WHERE o.comic_id = ?
        "#,
        *comic_id,
    )
    .fetch(&mut conn)
    .map_ok(|i| (i.id, i))
    .try_collect()
    .await
    .map_err(error::ErrorInternalServerError)?;

    let (comic_navigation_items, all_navigation_items) = if items.is_empty()
        && !matches!(query.include, Some(Inclusion::All))
    {
        (vec![], vec![])
    } else if let Some(Inclusion::All) = query.include {
        let mut all_items: BTreeMap<i16, DatabaseItem> = sqlx::query_as!(
            DatabaseItem,
            r#"
				SELECT *
				FROM items
			"#,
        )
        .fetch(&mut conn)
        .map_ok(|i| (i.id, i))
        .try_collect()
        .await
        .map_err(error::ErrorInternalServerError)?;

        let mut all_navigation_items =
            fetch_all_item_navigation_data(&mut conn, *comic_id, is_guest_comic, is_non_canon)
                .await?;
        let mut navigation_items_in_comic = vec![];
        let mut i = 0;
        while i < all_navigation_items.len() {
            let element = &mut all_navigation_items[i];
            if items.get(&element.id).is_some() {
                let element = all_navigation_items.remove(i);
                navigation_items_in_comic.push(element);
            } else {
                i += 1;
            }
        }

        transfer_item_data_to_navigation_item(&mut navigation_items_in_comic, &mut all_items)
            .map_err(error::ErrorInternalServerError)?;
        transfer_item_data_to_navigation_item(&mut all_navigation_items, &mut all_items)
            .map_err(error::ErrorInternalServerError)?;

        (navigation_items_in_comic, all_navigation_items)
    } else {
        let mut navigation_items_in_comic =
            fetch_comic_item_navigation_data(&mut conn, *comic_id, is_guest_comic, is_non_canon)
                .await?;

        transfer_item_data_to_navigation_item(&mut navigation_items_in_comic, &mut items)
            .map_err(error::ErrorInternalServerError)?;

        (navigation_items_in_comic, vec![])
    };

    let comic = if let Some(comic) = comic {
        Comic {
            comic: *comic_id,
            has_data: true,
            image_type: Some(comic.ImageType.into()),
            publish_date: comic.publishDate.map(|nd| DateTime::from_utc(nd, Utc)),
            is_accurate_publish_date: comic.isAccuratePublishDate != 0,
            title: Some(comic.title),
            tagline: comic.tagline,
            is_guest_comic: comic.isGuestComic != 0,
            is_non_canon: comic.isNonCanon != 0,
            has_no_cast: comic.HasNoCast != 0,
            has_no_location: comic.HasNoLocation != 0,
            has_no_storyline: comic.HasNoStoryline != 0,
            has_no_title: comic.HasNoTitle != 0,
            has_no_tagline: comic.HasNoTagline != 0,
            news: news.map(|n| n.news),
            previous,
            next,
            editor_data,
            items: comic_navigation_items,
            all_items: all_navigation_items,
        }
    } else {
        Comic {
            comic: *comic_id,
            has_data: false,
            image_type: None,
            publish_date: None,
            is_accurate_publish_date: false,
            title: None,
            tagline: None,
            is_guest_comic: false,
            is_non_canon: false,
            has_no_cast: false,
            has_no_location: false,
            has_no_storyline: false,
            has_no_title: false,
            has_no_tagline: false,
            news: None,
            previous,
            next,
            editor_data,
            items: comic_navigation_items,
            all_items: all_navigation_items,
        }
    };

    Ok(HttpResponse::Ok().json(comic))
}

#[allow(clippy::too_many_lines)]
async fn add_item(
    pool: web::Data<DbPool>,
    request: web::Json<AddItemToComicBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    const CREATE_NEW_ITEM_ID: i16 = -1;

    ensure_is_authorized(&auth, token_permissions::CAN_ADD_ITEM_TO_COMIC)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    ensure_comic_exists(&mut *transaction, request.comic_id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let item = if request.item_id == CREATE_NEW_ITEM_ID {
        let new_item_name = request.new_item_name.as_ref().ok_or_else(|| {
            error::ErrorBadRequest(anyhow!(
                "New Item request without providing newItemName value"
            ))
        })?;
        let new_item_type = request.new_item_type.as_ref().ok_or_else(|| {
            error::ErrorBadRequest(anyhow!(
                "New Item request without providing newItemType value"
            ))
        })?;

        let result = sqlx::query!(
            r#"
                INSERT INTO `items`
                    (name, shortName, type)
                VALUES
                    (?, ?, ?)
            "#,
            new_item_name,
            new_item_name,
            new_item_type,
        )
        .execute(&mut *transaction)
        .await
        .map_err(error::ErrorInternalServerError)?;

        let new_item_id = result.last_insert_id() as i16;

        log_action(
            &mut *transaction,
            request.token,
            format!(
                "Created {} #{} ({})",
                new_item_type, new_item_id, new_item_name
            ),
        )
        .await
        .map_err(error::ErrorInternalServerError)?;

        DatabaseItem {
            id: new_item_id,
            name: new_item_name.clone(),
            shortName: new_item_name.clone(),
            r#type: new_item_type.clone(),
            Color_Blue: 127,
            Color_Green: 127,
            Color_Red: 127,
        }
    } else {
        let item = sqlx::query_as!(
            DatabaseItem,
            r#"
                SELECT * FROM `items` WHERE id = ?
            "#,
            request.item_id
        )
        .fetch_optional(&mut *transaction)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorBadRequest(anyhow!("Item does not exist")))?;

        let occurrence_exists = sqlx::query_scalar!(
            r#"
                SELECT COUNT(1) FROM `occurences`
                WHERE
                    items_id = ?
                AND
                    comic_id = ?
            "#,
            request.item_id,
            request.comic_id,
        )
        .fetch_one(&mut *transaction)
        .await
        .map_err(error::ErrorInternalServerError)?
            == 1;

        if occurrence_exists {
            return Err(error::ErrorBadRequest(anyhow!(
                "Item is already added to comic"
            )));
        }

        item
    };

    sqlx::query!(
        r#"
            INSERT INTO `occurences`
                (comic_id, items_id)
            VALUES
                (?, ?)
        "#,
        request.comic_id,
        request.item_id
    )
    .execute(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?;

    log_action(
        &mut *transaction,
        request.token,
        format!(
            "Added {} #{} ({}) to comic #{}",
            item.r#type, item.id, item.name, request.comic_id
        ),
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    transaction
        .commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body("Item added to comic"))
}

async fn remove_item(
    pool: web::Data<DbPool>,
    request: web::Json<RemoveItemFromComicBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    ensure_is_authorized(&auth, token_permissions::CAN_REMOVE_ITEM_FROM_COMIC)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let comic_exists = sqlx::query_scalar!(
        r#"
            SELECT COUNT(1) FROM `comic`
            WHERE
                id = ?
        "#,
        request.comic_id,
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?
        == 1;

    if !comic_exists {
        return Err(error::ErrorBadRequest(anyhow!("Comic does not exist")));
    }

    let item = sqlx::query_as!(
        DatabaseItem,
        r#"
            SELECT * FROM `items` WHERE id = ?
        "#,
        request.item_id
    )
    .fetch_optional(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?
    .ok_or_else(|| error::ErrorBadRequest(anyhow!("Item does not exist")))?;

    sqlx::query!(
        r#"
            DELETE FROM `occurences`
            WHERE
                items_id = ?
            AND
                comic_id = ?
        "#,
        request.item_id,
        request.comic_id,
    )
    .execute(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?;

    log_action(
        &mut *transaction,
        request.token,
        format!(
            "Removed {} #{} ({}) from comic #{}",
            item.r#type, item.id, item.name, request.comic_id
        ),
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    transaction
        .commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body("Item removed from comic"))
}

async fn set_title(
    pool: web::Data<DbPool>,
    request: web::Json<SetTitleBody>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    ensure_is_authorized(&auth, token_permissions::CAN_CHANGE_COMIC_DATA)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    ensure_comic_exists(&mut *transaction, request.comic_id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let old_title = sqlx::query_scalar!(
        r#"
            SELECT title FROM `comic` WHERE id = ?
        "#,
        request.comic_id
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?;

    sqlx::query!(
        r#"
            UPDATE `comic`
            SET title = ?
            WHERE
                id = ?
        "#,
        request.title,
        request.comic_id,
    )
    .execute(&mut *transaction)
    .await
    .map_err(error::ErrorInternalServerError)?;

    if old_title.is_empty() {
        log_action(
            &mut *transaction,
            request.token,
            format!(
                "Set title on comic #{} to \"{}\"",
                request.comic_id, request.title
            ),
        )
        .await
        .map_err(error::ErrorInternalServerError)?;
    } else {
        log_action(
            &mut *transaction,
            request.token,
            format!(
                "Changed title on comic #{} from \"{}\" to \"{}\"",
                request.comic_id, old_title, request.title
            ),
        )
        .await
        .map_err(error::ErrorInternalServerError)?;
    }

    transaction
        .commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body("Title set or updated for comic"))
}

fn transfer_item_data_to_navigation_item(
    navigation_items: &mut Vec<crate::models::ItemNavigationData>,
    items: &mut BTreeMap<i16, DatabaseItem>,
) -> anyhow::Result<()> {
    for navigation_item in navigation_items {
        let DatabaseItem {
            id: _,
            shortName: short_name,
            name,
            r#type,
            Color_Red: color_red,
            Color_Green: color_green,
            Color_Blue: color_blue,
        } = items.remove(&navigation_item.id).unwrap();
        navigation_item.short_name = Some(short_name);
        navigation_item.name = Some(name);
        navigation_item.r#type = Some(ItemType::try_from(&*r#type)?);

        navigation_item.color = Some(ItemColor::new(color_red, color_green, color_blue));
    }

    Ok(())
}

async fn fetch_comic_list(
    pool: &DbPool,
    is_guest_comic: Option<bool>,
    is_non_canon: Option<bool>,
) -> Result<Vec<ComicList>> {
    let comics: Vec<ComicList> = sqlx::query_as!(
        DatabaseComic,
        r#"
			SELECT * FROM `comic`
			WHERE (? is NULL OR `isGuestComic` = ?)
				AND (? is NULL OR `isNonCanon` = ?)
			ORDER BY id ASC
		"#,
        is_guest_comic,
        is_guest_comic,
        is_non_canon,
        is_non_canon
    )
    .fetch(&**pool)
    .map_ok(From::from)
    .try_collect()
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(comics)
}

#[inline]
fn ensure_is_authorized(auth: &AuthDetails, permission: &str) -> Result<()> {
    if !auth.has_permission(permission) {
        return Err(error::ErrorForbidden(anyhow!(
            "Invalid token or insufficient permissions"
        )));
    }

    Ok(())
}

#[inline]
async fn ensure_comic_exists<'e, 'c: 'e, E>(executor: E, comic_id: i16) -> sqlx::Result<()>
where
    E: 'e + sqlx::Executor<'c, Database = sqlx::MySql>,
{
    sqlx::query!(
        r#"
            INSERT IGNORE INTO `comic`
                (id)
            VALUES
                (?)
        "#,
        comic_id,
    )
    .execute(executor)
    .await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct AllQuery {
    exclude: Option<Exclusion>,
}

#[derive(Debug, Deserialize)]
struct ExcludedQuery {
    exclusion: Option<Exclusion>,
}

#[derive(Debug, Deserialize)]
struct ByIdQuery {
    exclude: Option<Exclusion>,
    include: Option<Inclusion>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddItemToComicBody {
    token: uuid::Uuid,
    comic_id: i16,
    item_id: i16,
    #[serde(default)]
    new_item_name: Option<String>,
    #[serde(default)]
    new_item_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoveItemFromComicBody {
    token: uuid::Uuid,
    comic_id: i16,
    item_id: i16,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetTitleBody {
    token: uuid::Uuid,
    comic_id: i16,
    title: String,
}
