//! The public item and comic-search queries must give the same output with or without a hidden
//! (advance) comic in the database, and a different output once that comic is published.

use crate::models::hidden_comic_fixture::{
    CAST_A, CAST_B, CAST_NEW, HIDDEN_COMIC, hidden_comic_is_excluded, insert_hidden_comic,
    insert_occurrences, insert_visible_comics,
};
use crate::models::{Comic, Item, ItemType, Occurrence};
use sqlx::{MySqlConnection, MySqlPool};

const STORYLINE: u16 = 20;

hidden_comic_is_excluded!(item_all_with_counts, |conn| {
    let mut counts: Vec<(u16, i64)> = Item::all_with_counts(&mut *conn)
        .await?
        .iter()
        .map(|item| (item.id, item.count))
        .collect();
    counts.sort_unstable();
    counts
});
hidden_comic_is_excluded!(item_first_and_last_apperance_and_count_by_id, |conn| {
    Item::first_and_last_apperance_and_count_by_id(&mut *conn, CAST_B).await?
});
hidden_comic_is_excluded!(item_occurrence_stats_by_id_occurrences, |conn| {
    Item::occurrence_stats_by_id(&mut *conn, CAST_NEW).await?
});
hidden_comic_is_excluded!(item_occurrence_stats_by_id_total_comics, |conn| {
    Item::occurrence_stats_by_id(&mut *conn, CAST_A).await?
});
hidden_comic_is_excluded!(item_by_id_with_stats_occurrences, |conn| {
    Item::by_id_with_stats(&mut *conn, CAST_NEW).await?
});
hidden_comic_is_excluded!(item_by_id_with_stats_total_comics, |conn| {
    Item::by_id_with_stats(&mut *conn, CAST_A).await?
});
hidden_comic_is_excluded!(item_related_items_by_id_and_type, |conn| {
    let mut related: Vec<(u16, i64)> = Item::related_items_by_id_and_type_with_mapping(
        &mut *conn,
        CAST_B,
        ItemType::Cast,
        10,
        |item| (item.id, item.count),
    )
    .await?;
    related.sort_unstable();
    related
});
hidden_comic_is_excluded!(comic_all_with_item_id, |conn| {
    Comic::all_with_item_id_mapped(&mut *conn, CAST_B, |comic| comic.id).await?
});
hidden_comic_is_excluded!(occurrence_comic_ids_by_item_id, |conn| {
    let mut comic_ids = Occurrence::comic_id_occurrence_by_item_id(&mut *conn, CAST_B).await?;
    comic_ids.sort_unstable();
    comic_ids
});

#[sqlx::test]
async fn random_comic_with_item_never_picks_a_hidden_comic(pool: MySqlPool) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;
    insert_visible_comics(&mut conn).await?;
    insert_hidden_comic(&mut conn).await?;

    for _ in 0..20 {
        let comic = Comic::random_comic_id_with_item_id(&pool, CAST_NEW, 0, None, None).await?;
        assert_eq!(comic, None, "picked a hidden comic");
    }

    Comic::unhide_by_id(&mut *conn, HIDDEN_COMIC).await?;
    let comic = Comic::random_comic_id_with_item_id(&pool, CAST_NEW, 0, None, None).await?;
    assert_eq!(comic, Some(HIDDEN_COMIC));
    Ok(())
}

#[sqlx::test]
async fn active_storylines_leave_out_hidden_comics(pool: MySqlPool) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;
    insert_visible_comics(&mut conn).await?;
    insert_storyline(&mut conn).await?;
    let visible_only = active_storyline_comic_ids(&mut conn).await?;

    insert_hidden_comic(&mut conn).await?;
    insert_occurrences(&mut conn, HIDDEN_COMIC, &[STORYLINE]).await?;
    let with_hidden = active_storyline_comic_ids(&mut conn).await?;

    Comic::unhide_by_id(&mut *conn, HIDDEN_COMIC).await?;
    let published = active_storyline_comic_ids(&mut conn).await?;

    assert_eq!(visible_only, [Some(1)]);
    assert_eq!(
        with_hidden, visible_only,
        "the hidden comic changed the output"
    );
    assert_eq!(published, [Some(1), Some(HIDDEN_COMIC)]);
    Ok(())
}

/// An open-ended storyline starting at comic 1, attached to comic 1.
async fn insert_storyline(conn: &mut MySqlConnection) -> sqlx::Result<()> {
    sqlx::query(
        r"
            INSERT INTO `Item` (`id`, `short_name`, `name`, `type`, `start_comic_id`)
            VALUES (?, 'S', 'S', 'storyline', 1)
        ",
    )
    .bind(STORYLINE)
    .execute(&mut *conn)
    .await?;
    insert_occurrences(conn, 1, &[STORYLINE]).await
}

async fn active_storyline_comic_ids(conn: &mut MySqlConnection) -> sqlx::Result<Vec<Option<u16>>> {
    Ok(Item::active_storylines_by_comic_id(&mut *conn, 60)
        .await?
        .iter()
        .map(|row| row.occurrence_comic_id)
        .collect())
}
