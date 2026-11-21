//! Each stats query must give the same output with or without a hidden (advance) comic in the
//! database, and a different output once that comic is published.

use super::*;
use crate::models::Comic;
use crate::models::hidden_comic_fixture::{
    CAST_A, CAST_B, CAST_C, HIDDEN_COMIC, hidden_comic_is_excluded, insert_hidden_comic,
    insert_visible_comics,
};
use sqlx::MySqlPool;

hidden_comic_is_excluded!(item_stats_cast, |conn| ItemStats::cast(&mut *conn).await?);
hidden_comic_is_excluded!(item_stats_locations, |conn| ItemStats::locations(
    &mut *conn
)
.await?);
hidden_comic_is_excluded!(co_appearance_top, |conn| CoAppearance::top(&mut *conn)
    .await?);
hidden_comic_is_excluded!(co_appearance_top_normalized, |conn| {
    CoAppearance::top_normalized(&mut *conn).await?
});
hidden_comic_is_excluded!(yearly_appearance, |conn| YearlyAppearanceRow::all(
    &mut *conn
)
.await?);
hidden_comic_is_excluded!(debuts_per_year, |conn| DebutsPerYearRow::all(&mut *conn)
    .await?);
hidden_comic_is_excluded!(yearly_overview, |conn| YearlyOverviewRow::all(&mut *conn)
    .await?);
hidden_comic_is_excluded!(publication_month, |conn| PublicationMonthRow::all(
    &mut *conn
)
.await?);
hidden_comic_is_excluded!(publication_dow, |conn| PublicationDowRow::all(&mut *conn)
    .await?);
hidden_comic_is_excluded!(comeback_character, |conn| ComebackCharacterRow::top(
    &mut *conn
)
.await?);
hidden_comic_is_excluded!(comeback_location, |conn| ComebackLocationRow::top(
    &mut *conn
)
.await?);
hidden_comic_is_excluded!(location_affinity, |conn| LocationAffinityRow::all(
    &mut *conn
)
.await?);
hidden_comic_is_excluded!(crowded_comic, |conn| CrowdedComicRow::top(&mut *conn)
    .await?);
hidden_comic_is_excluded!(avg_cast_per_year, |conn| AvgCastPerYearRow::all(&mut *conn)
    .await?);
hidden_comic_is_excluded!(location_yearly_appearance, |conn| {
    LocationYearlyAppearanceRow::all(&mut *conn).await?
});
hidden_comic_is_excluded!(publication_gap, |conn| PublicationGapRow::top(&mut *conn)
    .await?);
hidden_comic_is_excluded!(debut_detail, |conn| DebutDetailRow::all(&mut *conn).await?);
hidden_comic_is_excluded!(ensemble_ratio, |conn| EnsembleRatioRow::all(&mut *conn)
    .await?);
hidden_comic_is_excluded!(character_regularity, |conn| {
    CharacterRegularityRow::all(&mut *conn).await?
});
hidden_comic_is_excluded!(location_regularity, |conn| LocationRegularityRow::all(
    &mut *conn
)
.await?);
hidden_comic_is_excluded!(location_co_occurrence, |conn| {
    LocationCoOccurrenceRow::top(&mut *conn).await?
});
hidden_comic_is_excluded!(social_hub, |conn| SocialHubRow::all(&mut *conn).await?);
hidden_comic_is_excluded!(cast_trending, |conn| TrendingItemRow::cast_trending(
    &mut *conn
)
.await?);
hidden_comic_is_excluded!(location_trending, |conn| {
    TrendingItemRow::location_trending(&mut *conn).await?
});
hidden_comic_is_excluded!(cast_turnover, |conn| CastTurnoverRow::all(&mut *conn)
    .await?);
hidden_comic_is_excluded!(character_season, |conn| CharacterSeasonRow::all(&mut *conn)
    .await?);
hidden_comic_is_excluded!(breakout_year, |conn| BreakoutYearRow::all(&mut *conn)
    .await?);
hidden_comic_is_excluded!(character_home_turf, |conn| CharacterHomeTurfRow::all(
    &mut *conn
)
.await?);
hidden_comic_is_excluded!(pair_evolution, |conn| PairEvolutionRow::for_pair(
    &mut *conn, CAST_B, CAST_C
)
.await?);
hidden_comic_is_excluded!(loner_index, |conn| LonerIndexRow::all(&mut *conn).await?);
hidden_comic_is_excluded!(never_met, |conn| NeverMetRow::all(&mut *conn).await?);
hidden_comic_is_excluded!(publish_time, |conn| PublishTimeRow::all(&mut *conn).await?);
hidden_comic_is_excluded!(schedule_evolution, |conn| ScheduleEvolutionRow::all(
    &mut *conn
)
.await?);
hidden_comic_is_excluded!(published_date, |conn| PublishedDateRow::all_distinct(
    &mut *conn
)
.await?);
hidden_comic_is_excluded!(monthly_heatmap, |conn| MonthlyHeatmapRow::all(&mut *conn)
    .await?);
hidden_comic_is_excluded!(milestone_comic, |conn| MilestoneComicRow::all(&mut *conn)
    .await?);
hidden_comic_is_excluded!(location_season, |conn| LocationSeasonRow::all(&mut *conn)
    .await?);
hidden_comic_is_excluded!(location_breakout_year, |conn| {
    LocationBreakoutYearRow::all(&mut *conn).await?
});
hidden_comic_is_excluded!(location_social_hub, |conn| LocationSocialHubRow::all(
    &mut *conn
)
.await?);
hidden_comic_is_excluded!(location_turnover, |conn| LocationTurnoverRow::all(
    &mut *conn
)
.await?);
hidden_comic_is_excluded!(latest_cast_comic_id, |conn| {
    TopRankedStintRow::latest_cast_comic_id(&mut *conn).await?
});
hidden_comic_is_excluded!(top_ranked_stints, |conn| {
    TopRankedStintRow::refresh_cache(&mut conn).await?;
    TopRankedStintRow::all(&mut *conn).await?
});

#[sqlx::test]
async fn changing_a_comics_hidden_flag_marks_rank_stints_stale(
    pool: MySqlPool,
) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;
    insert_visible_comics(&mut conn).await?;
    insert_hidden_comic(&mut conn).await?;

    TopRankedStintRow::refresh_cache(&mut conn).await?;
    assert!(!TopRankedStintRow::needs_refresh(&mut *conn).await?);

    Comic::unhide_by_id(&mut *conn, HIDDEN_COMIC).await?;
    assert!(
        TopRankedStintRow::needs_refresh(&mut *conn).await?,
        "publishing a comic did not mark the cache stale"
    );

    TopRankedStintRow::refresh_cache(&mut conn).await?;
    sqlx::query("UPDATE `Comic` SET `hidden` = 1 WHERE `id` = ?")
        .bind(HIDDEN_COMIC)
        .execute(&mut *conn)
        .await?;
    assert!(
        TopRankedStintRow::needs_refresh(&mut *conn).await?,
        "hiding a comic did not mark the cache stale"
    );

    TopRankedStintRow::refresh_cache(&mut conn).await?;
    sqlx::query("UPDATE `Comic` SET `title` = 'Renamed' WHERE `id` = ?")
        .bind(HIDDEN_COMIC)
        .execute(&mut *conn)
        .await?;
    assert!(
        !TopRankedStintRow::needs_refresh(&mut *conn).await?,
        "an update that leaves `hidden` alone marked the cache stale"
    );
    Ok(())
}

#[sqlx::test]
async fn cast_with_equal_appearances_is_ordered_by_id(pool: MySqlPool) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;
    insert_visible_comics(&mut conn).await?;

    let ids: Vec<u16> = ItemStats::cast(&mut *conn)
        .await?
        .iter()
        .map(|row| row.id)
        .collect();
    assert_eq!(ids, [CAST_A, CAST_B, CAST_C]);
    Ok(())
}

#[sqlx::test]
async fn pairs_with_equal_comics_together_are_ordered_by_ids(pool: MySqlPool) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;
    insert_visible_comics(&mut conn).await?;

    let pairs: Vec<(u16, u16)> = CoAppearance::top(&mut *conn)
        .await?
        .iter()
        .map(|row| (row.character1_id, row.character2_id))
        .collect();
    assert_eq!(
        pairs,
        [(CAST_A, CAST_B), (CAST_A, CAST_C), (CAST_B, CAST_C)]
    );
    Ok(())
}

#[sqlx::test]
async fn social_hubs_with_equal_partners_are_ordered_by_id(pool: MySqlPool) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;
    insert_visible_comics(&mut conn).await?;

    let ids: Vec<u16> = SocialHubRow::all(&mut *conn)
        .await?
        .iter()
        .map(|row| row.id)
        .collect();
    assert_eq!(ids, [CAST_A, CAST_B, CAST_C]);
    Ok(())
}
