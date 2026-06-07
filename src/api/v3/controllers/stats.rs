use std::collections::HashMap;

use crate::api::v3::models::ItemColor;
use crate::api::v3::models::stats::{
    AvgCastPerYear, BestFriendPair, BestFriendResponse, BreakoutYear, CastTurnoverYear,
    CharacterHomeTurfEntry, CharacterMeta, CharacterRegularity, CharacterSeasonEntry,
    CoAppearanceCharacterMeta, CoAppearancePair, CoAppearancesResponse, ComebackCharacter,
    CrowdedComicsResponse, DailyComics, DebutCharacter, DebutYear, DebutsPerYear, EnsembleRatio,
    ItemStats, LocationAffinity, LocationAffinityCharacter, LocationCoOccurrenceEntry,
    LocationCoOccurrencePair, LocationCoOccurrenceResponse, LocationSpotlightResponse,
    LocationSpotlightYear, LonerEntry, MilestoneComic, MonthlyComics, MonthlyHeatmapEntry,
    MostCrowdedComic, NeverMetPair, PairEvolutionYear, PublicationCalendar, PublicationGap,
    PublicationStreak, PublishTimeYear, ScheduleEvolutionYear, SocialHubEntry, TrendingItem,
    YearlyOverview, YearlyRankEntry, YearlySpotlightResponse, YearlySpotlightYear,
};
use crate::models::{ComicId, ItemId};
use crate::util::environment;
use actix_web::{HttpResponse, Result, error, web};
use database::DbPool;
use database::models::stats::{
    AvgCastPerYearRow as DbAvgCastPerYearRow, BreakoutYearRow as DbBreakoutYearRow,
    CastTurnoverRow as DbCastTurnoverRow, CharacterHomeTurfRow as DbCharacterHomeTurfRow,
    CharacterRegularityRow as DbCharacterRegularityRow, CharacterSeasonRow as DbCharacterSeasonRow,
    CoAppearance as DbCoAppearance, ComebackCharacterRow as DbComebackCharacterRow,
    CrowdedComicRow as DbCrowdedComicRow, DebutDetailRow as DbDebutDetailRow,
    DebutsPerYearRow as DbDebutsPerYearRow, EnsembleRatioRow as DbEnsembleRatioRow,
    ItemStats as DbItemStats, LocationAffinityRow as DbLocationAffinityRow,
    LocationCoOccurrenceRow as DbLocationCoOccurrenceRow,
    LocationYearlyAppearanceRow as DbLocationYearlyAppearanceRow, LonerIndexRow as DbLonerIndexRow,
    MilestoneComicRow as DbMilestoneComicRow, MonthlyHeatmapRow as DbMonthlyHeatmapRow,
    NeverMetRow as DbNeverMetRow, PairEvolutionRow as DbPairEvolutionRow,
    PublicationDowRow as DbPublicationDowRow, PublicationGapRow as DbPublicationGapRow,
    PublicationMonthRow as DbPublicationMonthRow, PublishTimeRow as DbPublishTimeRow,
    PublishedDateRow as DbPublishedDateRow, ScheduleEvolutionRow as DbScheduleEvolutionRow,
    SocialHubRow as DbSocialHubRow, TrendingItemRow as DbTrendingItemRow,
    YearlyAppearanceRow as DbYearlyAppearanceRow, YearlyOverviewRow as DbYearlyOverviewRow,
};
use serde::Deserialize;
use tracing::{Instrument, info_span};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/cast").route(web::get().to(cast)))
        .service(web::resource("/locations").route(web::get().to(locations)))
        .service(web::resource("/co-appearances").route(web::get().to(co_appearances)))
        .service(web::resource("/yearly-spotlight").route(web::get().to(yearly_spotlight)))
        .service(web::resource("/debuts-per-year").route(web::get().to(debuts_per_year)))
        .service(web::resource("/yearly-overview").route(web::get().to(yearly_overview)))
        .service(web::resource("/publication-calendar").route(web::get().to(publication_calendar)))
        .service(web::resource("/comeback-characters").route(web::get().to(comeback_characters)))
        .service(web::resource("/location-affinity").route(web::get().to(location_affinity)))
        .service(web::resource("/crowded-comics").route(web::get().to(crowded_comics)))
        .service(
            web::resource("/location-yearly-spotlight")
                .route(web::get().to(location_yearly_spotlight)),
        )
        .service(web::resource("/publication-gaps").route(web::get().to(publication_gaps)))
        .service(web::resource("/debut-clusters").route(web::get().to(debut_clusters)))
        .service(web::resource("/ensemble-ratio").route(web::get().to(ensemble_ratio)))
        .service(web::resource("/character-regularity").route(web::get().to(character_regularity)))
        .service(
            web::resource("/location-co-occurrences").route(web::get().to(location_co_occurrences)),
        )
        .service(web::resource("/best-friend-score").route(web::get().to(best_friend_score)))
        .service(web::resource("/social-hub").route(web::get().to(social_hub)))
        .service(web::resource("/trending-characters").route(web::get().to(trending_characters)))
        .service(web::resource("/trending-locations").route(web::get().to(trending_locations)))
        .service(web::resource("/cast-turnover").route(web::get().to(cast_turnover)))
        .service(web::resource("/character-seasons").route(web::get().to(character_seasons)))
        .service(web::resource("/breakout-years").route(web::get().to(breakout_years)))
        .service(web::resource("/character-home-turf").route(web::get().to(character_home_turf)))
        .service(web::resource("/pair-evolution").route(web::get().to(pair_evolution)))
        .service(web::resource("/loner-index").route(web::get().to(loner_index)))
        .service(web::resource("/never-met").route(web::get().to(never_met)))
        .service(web::resource("/schedule-evolution").route(web::get().to(schedule_evolution)))
        .service(
            web::resource("/publish-time-evolution").route(web::get().to(publish_time_evolution)),
        )
        .service(web::resource("/publication-streaks").route(web::get().to(publication_streaks)))
        .service(web::resource("/monthly-heatmap").route(web::get().to(monthly_heatmap)))
        .service(web::resource("/milestones").route(web::get().to(milestones)));
}

#[tracing::instrument(skip(pool))]
async fn cast(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbItemStats::cast(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(rows.into_iter().map(item_stats_from_db).collect::<Vec<_>>()))
}

#[tracing::instrument(skip(pool))]
async fn locations(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbItemStats::locations(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(rows.into_iter().map(item_stats_from_db).collect::<Vec<_>>()))
}

#[tracing::instrument(skip(pool))]
async fn co_appearances(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbCoAppearance::top(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(build_co_appearances_response(rows)))
}

#[tracing::instrument(skip(pool))]
async fn yearly_spotlight(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbYearlyAppearanceRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(build_yearly_spotlight_response(rows)))
}

#[tracing::instrument(skip(pool))]
async fn debuts_per_year(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbDebutsPerYearRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let result: Vec<DebutsPerYear> = rows
        .into_iter()
        .filter_map(|row| {
            Some(DebutsPerYear {
                year: row.year?,
                cast_debuts: u32::try_from(row.cast_debuts).unwrap_or(u32::MAX),
                location_debuts: u32::try_from(row.location_debuts).unwrap_or(u32::MAX),
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(result))
}

#[tracing::instrument(skip(pool))]
async fn yearly_overview(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbYearlyOverviewRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let result: Vec<YearlyOverview> = rows
        .into_iter()
        .filter_map(|row| {
            Some(YearlyOverview {
                year: row.year?,
                total_cast: u32::try_from(row.total_cast).unwrap_or(u32::MAX),
                new_cast: u32::try_from(row.new_cast).unwrap_or(u32::MAX),
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(result))
}

#[tracing::instrument(skip(pool))]
async fn publication_calendar(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let month_rows = DbPublicationMonthRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let dow_rows = DbPublicationDowRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let monthly: Vec<MonthlyComics> = month_rows
        .into_iter()
        .filter_map(|row| {
            Some(MonthlyComics {
                month: u8::try_from(row.month?).ok()?,
                comics: u32::try_from(row.comics).unwrap_or(u32::MAX),
            })
        })
        .collect();

    let daily: Vec<DailyComics> = dow_rows
        .into_iter()
        .filter_map(|row| {
            Some(DailyComics {
                dow: u8::try_from(row.dow?).ok()?,
                comics: u32::try_from(row.comics).unwrap_or(u32::MAX),
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(PublicationCalendar { monthly, daily }))
}

#[tracing::instrument(skip(pool))]
async fn comeback_characters(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbComebackCharacterRow::top(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let result: Vec<ComebackCharacter> = rows
        .into_iter()
        .filter_map(|row| {
            Some(ComebackCharacter {
                id: ItemId::from(row.id),
                name: row.name,
                last_comic: ComicId::from_trusted(row.last_comic?),
                return_comic: ComicId::from_trusted(row.return_comic?),
                gap_days: u32::try_from(row.gap_days?).unwrap_or(u32::MAX),
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(result))
}

#[tracing::instrument(skip(pool))]
async fn location_affinity(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbLocationAffinityRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(build_location_affinity_response(rows)))
}

#[tracing::instrument(skip(pool))]
async fn crowded_comics(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let comic_rows = DbCrowdedComicRow::top(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let avg_rows = DbAvgCastPerYearRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let top_comics: Vec<MostCrowdedComic> = comic_rows
        .into_iter()
        .map(|row| MostCrowdedComic {
            comic_id: ComicId::from_trusted(row.comic_id),
            cast_count: u32::try_from(row.cast_count).unwrap_or(u32::MAX),
        })
        .collect();

    let avg_per_year: Vec<AvgCastPerYear> = avg_rows
        .into_iter()
        .filter_map(|row| {
            Some(AvgCastPerYear {
                year: row.year?,
                avg_cast_size: row.avg_cast_size?,
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(CrowdedComicsResponse {
        top_comics,
        avg_per_year,
    }))
}

#[tracing::instrument(skip(pool))]
async fn location_yearly_spotlight(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbLocationYearlyAppearanceRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(build_location_spotlight_response(rows)))
}

#[tracing::instrument(skip(pool))]
async fn publication_gaps(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbPublicationGapRow::top(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let result: Vec<PublicationGap> = rows
        .into_iter()
        .filter_map(|row| {
            Some(PublicationGap {
                before_comic: ComicId::from_trusted(row.before_comic?),
                after_comic: ComicId::from_trusted(row.after_comic?),
                gap_days: u32::try_from(row.gap_days?).unwrap_or(u32::MAX),
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(result))
}

#[tracing::instrument(skip(pool))]
async fn debut_clusters(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbDebutDetailRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(build_debut_clusters_response(rows)))
}

#[tracing::instrument(skip(pool))]
async fn ensemble_ratio(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbEnsembleRatioRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let result: Vec<EnsembleRatio> = rows
        .into_iter()
        .filter_map(|row| {
            Some(EnsembleRatio {
                year: row.year?,
                no_cast: u32::try_from(row.no_cast).unwrap_or(u32::MAX),
                solo: u32::try_from(row.solo).unwrap_or(u32::MAX),
                small_group: u32::try_from(row.small_group).unwrap_or(u32::MAX),
                large_group: u32::try_from(row.large_group).unwrap_or(u32::MAX),
                total: u32::try_from(row.total).unwrap_or(u32::MAX),
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(result))
}

#[tracing::instrument(skip(pool))]
async fn character_regularity(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbCharacterRegularityRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let result: Vec<CharacterRegularity> = rows
        .into_iter()
        .filter_map(|row| {
            Some(CharacterRegularity {
                id: ItemId::from(row.id),
                name: row.name,
                appearances: u32::try_from(row.gap_count + 1).unwrap_or(u32::MAX),
                avg_gap_days: row.avg_gap_days?,
                stddev_gap_days: row.stddev_gap_days?,
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(result))
}

#[tracing::instrument(skip(pool))]
async fn location_co_occurrences(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbLocationCoOccurrenceRow::top(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(build_location_co_occurrences_response(rows)))
}

#[tracing::instrument(skip(pool))]
async fn best_friend_score(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbCoAppearance::top_normalized(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(build_best_friend_response(rows)))
}

fn item_stats_from_db(row: DbItemStats) -> Option<ItemStats> {
    let (Some(first_comic), Some(last_comic)) = (row.first_comic, row.last_comic) else {
        return None;
    };
    Some(ItemStats {
        id: ItemId::from(row.id),
        name: row.name,
        first_comic: ComicId::from_trusted(first_comic),
        last_comic: ComicId::from_trusted(last_comic),
        appearances: u32::try_from(row.appearances).unwrap_or(u32::MAX),
    })
}

fn build_co_appearances_response(rows: Vec<DbCoAppearance>) -> CoAppearancesResponse {
    let mut characters: HashMap<u16, CoAppearanceCharacterMeta> = HashMap::new();
    let mut pairs = Vec::with_capacity(rows.len());

    for row in rows {
        characters
            .entry(row.character1_id)
            .or_insert_with(|| CoAppearanceCharacterMeta {
                name: row.character1_name.clone(),
                appearances: u32::try_from(row.character1_appearances).unwrap_or(u32::MAX),
            });
        characters
            .entry(row.character2_id)
            .or_insert_with(|| CoAppearanceCharacterMeta {
                name: row.character2_name.clone(),
                appearances: u32::try_from(row.character2_appearances).unwrap_or(u32::MAX),
            });
        pairs.push(CoAppearancePair {
            character1_id: ItemId::from(row.character1_id),
            character2_id: ItemId::from(row.character2_id),
            comics_together: u32::try_from(row.comics_together).unwrap_or(u32::MAX),
        });
    }

    CoAppearancesResponse { characters, pairs }
}

fn build_yearly_spotlight_response(rows: Vec<DbYearlyAppearanceRow>) -> YearlySpotlightResponse {
    let mut characters: HashMap<u16, CharacterMeta> = HashMap::new();
    let mut years: Vec<YearlySpotlightYear> = Vec::new();

    for row in rows {
        let Some(year) = row.year else { continue };

        match years.last_mut() {
            Some(spotlight) if spotlight.year == year => {
                if spotlight.characters.len() < 5 {
                    characters.entry(row.id).or_insert_with(|| CharacterMeta {
                        name: row.name,
                        color: ItemColor(row.color_red, row.color_green, row.color_blue),
                    });
                    spotlight.characters.push(YearlyRankEntry {
                        id: ItemId::from(row.id),
                        appearances: u32::try_from(row.appearances).unwrap_or(u32::MAX),
                    });
                }
            }
            _ => {
                characters.entry(row.id).or_insert_with(|| CharacterMeta {
                    name: row.name,
                    color: ItemColor(row.color_red, row.color_green, row.color_blue),
                });
                years.push(YearlySpotlightYear {
                    year,
                    characters: vec![YearlyRankEntry {
                        id: ItemId::from(row.id),
                        appearances: u32::try_from(row.appearances).unwrap_or(u32::MAX),
                    }],
                });
            }
        }
    }

    YearlySpotlightResponse { characters, years }
}

fn build_location_spotlight_response(
    rows: Vec<DbLocationYearlyAppearanceRow>,
) -> LocationSpotlightResponse {
    let mut locations: HashMap<u16, CharacterMeta> = HashMap::new();
    let mut years: Vec<LocationSpotlightYear> = Vec::new();

    for row in rows {
        let Some(year) = row.year else { continue };

        match years.last_mut() {
            Some(spotlight) if spotlight.year == year => {
                if spotlight.locations.len() < 5 {
                    locations.entry(row.id).or_insert_with(|| CharacterMeta {
                        name: row.name,
                        color: ItemColor(row.color_red, row.color_green, row.color_blue),
                    });
                    spotlight.locations.push(YearlyRankEntry {
                        id: ItemId::from(row.id),
                        appearances: u32::try_from(row.appearances).unwrap_or(u32::MAX),
                    });
                }
            }
            _ => {
                locations.entry(row.id).or_insert_with(|| CharacterMeta {
                    name: row.name,
                    color: ItemColor(row.color_red, row.color_green, row.color_blue),
                });
                years.push(LocationSpotlightYear {
                    year,
                    locations: vec![YearlyRankEntry {
                        id: ItemId::from(row.id),
                        appearances: u32::try_from(row.appearances).unwrap_or(u32::MAX),
                    }],
                });
            }
        }
    }

    LocationSpotlightResponse { locations, years }
}

fn build_debut_clusters_response(rows: Vec<DbDebutDetailRow>) -> Vec<DebutYear> {
    let mut years: Vec<DebutYear> = Vec::new();

    for row in rows {
        let Some(year) = row.year else { continue };

        match years.last_mut() {
            Some(dy) if dy.year == year => {
                dy.characters.push(DebutCharacter {
                    id: ItemId::from(row.id),
                    name: row.name,
                });
            }
            _ => {
                years.push(DebutYear {
                    year,
                    characters: vec![DebutCharacter {
                        id: ItemId::from(row.id),
                        name: row.name,
                    }],
                });
            }
        }
    }

    years
}

fn build_location_co_occurrences_response(
    rows: Vec<DbLocationCoOccurrenceRow>,
) -> LocationCoOccurrenceResponse {
    let mut locations: HashMap<u16, LocationCoOccurrenceEntry> = HashMap::new();
    let mut pairs = Vec::with_capacity(rows.len());

    for row in rows {
        locations
            .entry(row.location1_id)
            .or_insert_with(|| LocationCoOccurrenceEntry {
                id: ItemId::from(row.location1_id),
                name: row.location1_name.clone(),
                appearances: u32::try_from(row.location1_appearances).unwrap_or(u32::MAX),
            });
        locations
            .entry(row.location2_id)
            .or_insert_with(|| LocationCoOccurrenceEntry {
                id: ItemId::from(row.location2_id),
                name: row.location2_name.clone(),
                appearances: u32::try_from(row.location2_appearances).unwrap_or(u32::MAX),
            });
        pairs.push(LocationCoOccurrencePair {
            location1_id: ItemId::from(row.location1_id),
            location2_id: ItemId::from(row.location2_id),
            comics_together: u32::try_from(row.comics_together).unwrap_or(u32::MAX),
        });
    }

    LocationCoOccurrenceResponse { locations, pairs }
}

fn build_best_friend_response(rows: Vec<DbCoAppearance>) -> BestFriendResponse {
    let mut characters: HashMap<u16, CoAppearanceCharacterMeta> = HashMap::new();
    let mut pairs = Vec::with_capacity(rows.len());

    for row in rows {
        characters
            .entry(row.character1_id)
            .or_insert_with(|| CoAppearanceCharacterMeta {
                name: row.character1_name.clone(),
                appearances: u32::try_from(row.character1_appearances).unwrap_or(u32::MAX),
            });
        characters
            .entry(row.character2_id)
            .or_insert_with(|| CoAppearanceCharacterMeta {
                name: row.character2_name.clone(),
                appearances: u32::try_from(row.character2_appearances).unwrap_or(u32::MAX),
            });
        pairs.push(BestFriendPair {
            character1_id: ItemId::from(row.character1_id),
            character2_id: ItemId::from(row.character2_id),
            comics_together: u32::try_from(row.comics_together).unwrap_or(u32::MAX),
        });
    }

    BestFriendResponse { characters, pairs }
}

#[tracing::instrument(skip(pool))]
async fn social_hub(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbSocialHubRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let result: Vec<SocialHubEntry> = rows
        .into_iter()
        .map(|r| SocialHubEntry {
            id: ItemId::from(r.id),
            name: r.name,
            appearances: u32::try_from(r.appearances).unwrap_or(u32::MAX),
            distinct_partners: u32::try_from(r.distinct_partners).unwrap_or(u32::MAX),
        })
        .collect();
    Ok(HttpResponse::Ok().json(result))
}

#[tracing::instrument(skip(pool))]
async fn trending_characters(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbTrendingItemRow::cast_trending(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(build_trending_response(rows)))
}

#[tracing::instrument(skip(pool))]
async fn trending_locations(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbTrendingItemRow::location_trending(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(build_trending_response(rows)))
}

#[tracing::instrument(skip(pool))]
async fn cast_turnover(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbCastTurnoverRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let result: Vec<CastTurnoverYear> = rows
        .into_iter()
        .filter_map(|r| {
            Some(CastTurnoverYear {
                year: r.year?,
                new_chars: u32::try_from(r.new_chars).unwrap_or(u32::MAX),
                continuing_chars: u32::try_from(r.continuing_chars).unwrap_or(u32::MAX),
                returning_chars: u32::try_from(r.returning_chars).unwrap_or(u32::MAX),
                dropped_chars: u32::try_from(r.dropped_chars.unwrap_or(0)).unwrap_or(u32::MAX),
            })
        })
        .collect();
    Ok(HttpResponse::Ok().json(result))
}

#[tracing::instrument(skip(pool))]
async fn character_seasons(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbCharacterSeasonRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(build_character_seasons_response(rows)))
}

#[tracing::instrument(skip(pool))]
async fn breakout_years(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbBreakoutYearRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let result: Vec<BreakoutYear> = rows
        .into_iter()
        .filter_map(|r| {
            let avg = r.avg_per_year.unwrap_or(0.0);
            let breakout_years = r
                .breakout_years?
                .split(',')
                .filter_map(|s| s.parse::<i32>().ok())
                .collect();
            Some(BreakoutYear {
                id: ItemId::from(r.id),
                name: r.name,
                breakout_years,
                breakout_count: u32::try_from(r.breakout_count).unwrap_or(u32::MAX),
                avg_per_year: avg,
                #[expect(clippy::cast_precision_loss, reason = "ratio is approximate by nature")]
                ratio: if avg > 0.0 {
                    r.breakout_count as f64 / avg
                } else {
                    0.0
                },
            })
        })
        .collect();
    Ok(HttpResponse::Ok().json(result))
}

#[tracing::instrument(skip(pool))]
async fn character_home_turf(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbCharacterHomeTurfRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let result: Vec<CharacterHomeTurfEntry> = rows
        .into_iter()
        .map(|r| CharacterHomeTurfEntry {
            character_id: ItemId::from(r.character_id),
            character_name: r.character_name,
            location_id: ItemId::from(r.location_id),
            location_name: r.location_name,
            comics_together: u32::try_from(r.comics_together).unwrap_or(u32::MAX),
            character_appearances: u32::try_from(r.character_appearances).unwrap_or(u32::MAX),
        })
        .collect();
    Ok(HttpResponse::Ok().json(result))
}

#[derive(Debug, Deserialize)]
struct PairEvolutionQuery {
    char1: u16,
    char2: u16,
}

#[tracing::instrument(skip(pool))]
async fn pair_evolution(
    pool: web::Data<DbPool>,
    query: web::Query<PairEvolutionQuery>,
) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbPairEvolutionRow::for_pair(&mut *conn, query.char1, query.char2)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let result: Vec<PairEvolutionYear> = rows
        .into_iter()
        .filter_map(|r| {
            Some(PairEvolutionYear {
                year: r.year?,
                comics_together: u32::try_from(r.comics_together).unwrap_or(u32::MAX),
            })
        })
        .collect();
    Ok(HttpResponse::Ok().json(result))
}

#[tracing::instrument(skip(pool))]
async fn loner_index(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbLonerIndexRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let result: Vec<LonerEntry> = rows
        .into_iter()
        .map(|r| LonerEntry {
            id: ItemId::from(r.id),
            name: r.name,
            appearances: u32::try_from(r.appearances).unwrap_or(u32::MAX),
            avg_co_cast: r.avg_co_cast.unwrap_or(0.0),
        })
        .collect();
    Ok(HttpResponse::Ok().json(result))
}

#[tracing::instrument(skip(pool))]
async fn never_met(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbNeverMetRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let result: Vec<NeverMetPair> = rows
        .into_iter()
        .map(|r| NeverMetPair {
            character1_id: ItemId::from(r.character1_id),
            character1_name: r.character1_name,
            character1_appearances: u32::try_from(r.character1_appearances).unwrap_or(u32::MAX),
            character2_id: ItemId::from(r.character2_id),
            character2_name: r.character2_name,
            character2_appearances: u32::try_from(r.character2_appearances).unwrap_or(u32::MAX),
            comics_together: u32::try_from(r.comics_together.unwrap_or(0)).unwrap_or(u32::MAX),
        })
        .collect();
    Ok(HttpResponse::Ok().json(result))
}

#[tracing::instrument(skip(pool))]
async fn schedule_evolution(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbScheduleEvolutionRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(build_schedule_evolution_response(rows)))
}

#[tracing::instrument(skip(pool))]
async fn publish_time_evolution(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbPublishTimeRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(build_publish_time_response(rows)))
}

#[tracing::instrument(skip(pool))]
async fn publication_streaks(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbPublishedDateRow::all_distinct(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let dates: Vec<String> = rows.into_iter().filter_map(|r| r.pub_date).collect();
    Ok(HttpResponse::Ok().json(build_publication_streaks(&dates)))
}

#[tracing::instrument(skip(pool))]
async fn monthly_heatmap(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbMonthlyHeatmapRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let result: Vec<MonthlyHeatmapEntry> = rows
        .into_iter()
        .filter_map(|r| {
            Some(MonthlyHeatmapEntry {
                year: r.year?,
                month: u8::try_from(r.month?).ok()?,
                comics: u32::try_from(r.comics).unwrap_or(u32::MAX),
            })
        })
        .collect();
    Ok(HttpResponse::Ok().json(result))
}

#[tracing::instrument(skip(pool))]
async fn milestones(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool
        .acquire()
        .instrument(info_span!("Pool::acquire"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let rows = DbMilestoneComicRow::all(&mut *conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let result: Vec<MilestoneComic> = rows
        .into_iter()
        .map(|r| MilestoneComic {
            comic_id: ComicId::from_trusted(r.comic_id),
            title: r.title,
            pub_date: r.pub_date,
            is_guest_comic: r.is_guest_comic != 0,
            is_non_canon: r.is_non_canon != 0,
        })
        .collect();
    Ok(HttpResponse::Ok().json(result))
}

fn build_trending_response(rows: Vec<DbTrendingItemRow>) -> Vec<TrendingItem> {
    rows.into_iter()
        .map(|r| TrendingItem {
            id: ItemId::from(r.id),
            name: r.name,
            total_appearances: u32::try_from(r.total_appearances).unwrap_or(u32::MAX),
            recent_appearances: u32::try_from(r.recent_appearances).unwrap_or(u32::MAX),
            career_years: r.career_years.unwrap_or(0.0),
        })
        .collect()
}

fn build_character_seasons_response(rows: Vec<DbCharacterSeasonRow>) -> Vec<CharacterSeasonEntry> {
    let mut entries: Vec<CharacterSeasonEntry> = Vec::new();
    for row in rows {
        let Some(month) = row.month else { continue };
        let month_idx = usize::try_from(month - 1).unwrap_or(0);
        if month_idx >= 12 {
            continue;
        }
        match entries.last_mut() {
            Some(e) if e.id.into_inner() == row.id => {
                e.monthly[month_idx] = u32::try_from(row.appearances).unwrap_or(u32::MAX);
            }
            _ => {
                let mut monthly = vec![0_u32; 12];
                monthly[month_idx] = u32::try_from(row.appearances).unwrap_or(u32::MAX);
                entries.push(CharacterSeasonEntry {
                    id: ItemId::from(row.id),
                    name: row.name,
                    monthly,
                });
            }
        }
    }
    entries
}

fn build_publish_time_response(rows: Vec<DbPublishTimeRow>) -> Vec<PublishTimeYear> {
    let mut years: Vec<PublishTimeYear> = Vec::new();
    for row in rows {
        let (Some(year), Some(hour)) = (row.year, row.hour) else {
            continue;
        };
        let hour_idx = usize::try_from(hour).unwrap_or(0);
        if hour_idx >= 24 {
            continue;
        }
        match years.last_mut() {
            Some(y) if y.year == year => {
                y.hour_counts[hour_idx] = u32::try_from(row.comics).unwrap_or(u32::MAX);
            }
            _ => {
                let mut hour_counts = vec![0_u32; 24];
                hour_counts[hour_idx] = u32::try_from(row.comics).unwrap_or(u32::MAX);
                years.push(PublishTimeYear { year, hour_counts });
            }
        }
    }
    years
}

fn build_schedule_evolution_response(
    rows: Vec<DbScheduleEvolutionRow>,
) -> Vec<ScheduleEvolutionYear> {
    let mut years: Vec<ScheduleEvolutionYear> = Vec::new();
    for row in rows {
        let (Some(year), Some(dow)) = (row.year, row.dow) else {
            continue;
        };
        // DAYOFWEEK: 1=Sun…7=Sat; remap so Mon=0…Sun=6
        let dow_idx = usize::try_from((dow + 5) % 7).unwrap_or(0);
        if dow_idx >= 7 {
            continue;
        }
        match years.last_mut() {
            Some(y) if y.year == year => {
                y.dow_counts[dow_idx] = u32::try_from(row.comics).unwrap_or(u32::MAX);
            }
            _ => {
                let mut dow_counts = vec![0_u32; 7];
                dow_counts[dow_idx] = u32::try_from(row.comics).unwrap_or(u32::MAX);
                years.push(ScheduleEvolutionYear { year, dow_counts });
            }
        }
    }
    years
}

#[expect(
    clippy::many_single_char_names,
    reason = "julian day formula uses math convention"
)]
fn julian_day(s: &str) -> Option<u32> {
    let mut parts = s.splitn(3, '-');
    let y: u32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    let d: u32 = parts.next()?.parse().ok()?;
    let a = (14 - m) / 12;
    let yr = y + 4800 - a;
    let mo = m + 12 * a - 3;
    Some(d + (153 * mo + 2) / 5 + 365 * yr + yr / 4 - yr / 100 + yr / 400 - 32_045)
}

fn date_gap_days(date_a: &str, date_b: &str) -> u32 {
    match (julian_day(date_a), julian_day(date_b)) {
        (Some(ja), Some(jb)) => jb.saturating_sub(ja),
        _ => u32::MAX,
    }
}

fn is_consecutive_weekday_publication(date_a: &str, date_b: &str) -> bool {
    match (julian_day(date_a), julian_day(date_b)) {
        (Some(ja), Some(jb)) => {
            // JDN % 7: 0=Mon, 1=Tue, 2=Wed, 3=Thu, 4=Fri, 5=Sat, 6=Sun
            // Friday wraps to Monday with a 3-day calendar gap; all other weekdays advance by 1
            let expected = if ja % 7 == 4 { ja + 3 } else { ja + 1 };
            jb == expected
        }
        _ => false,
    }
}

fn build_publication_streaks(dates: &[String]) -> Vec<PublicationStreak> {
    if dates.is_empty() {
        return Vec::new();
    }

    let mut streaks: Vec<PublicationStreak> = Vec::new();
    let mut streak_start = dates[0].clone();
    let mut streak_end = dates[0].clone();
    let mut days_in_streak: u32 = 1;

    for i in 1..dates.len() {
        if is_consecutive_weekday_publication(&dates[i - 1], &dates[i]) {
            streak_end.clone_from(&dates[i]);
            days_in_streak += 1;
        } else {
            let calendar_days = date_gap_days(&streak_start, &streak_end) + 1;
            streaks.push(PublicationStreak {
                streak_start: streak_start.clone(),
                streak_end: streak_end.clone(),
                days_with_comics: days_in_streak,
                calendar_days,
            });
            streak_start.clone_from(&dates[i]);
            streak_end.clone_from(&dates[i]);
            days_in_streak = 1;
        }
    }
    let calendar_days = date_gap_days(&streak_start, &streak_end) + 1;
    streaks.push(PublicationStreak {
        streak_start,
        streak_end,
        days_with_comics: days_in_streak,
        calendar_days,
    });

    streaks.sort_by_key(|s| std::cmp::Reverse(s.days_with_comics));
    streaks.truncate(20);
    streaks
}

fn build_location_affinity_response(rows: Vec<DbLocationAffinityRow>) -> Vec<LocationAffinity> {
    let mut locations: Vec<LocationAffinity> = Vec::new();

    for row in rows {
        match locations.last_mut() {
            Some(loc) if loc.location_id.into_inner() == row.location_id => {
                if loc.top_characters.len() < 5 {
                    loc.top_characters.push(LocationAffinityCharacter {
                        id: ItemId::from(row.character_id),
                        name: row.character_name,
                        comics_together: u32::try_from(row.comics_together).unwrap_or(u32::MAX),
                    });
                }
            }
            _ => {
                locations.push(LocationAffinity {
                    location_id: ItemId::from(row.location_id),
                    location_name: row.location_name,
                    top_characters: vec![LocationAffinityCharacter {
                        id: ItemId::from(row.character_id),
                        name: row.character_name,
                        comics_together: u32::try_from(row.comics_together).unwrap_or(u32::MAX),
                    }],
                });
            }
        }
    }

    locations
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_co_appearance(
        id1: u16,
        name1: &str,
        app1: i64,
        id2: u16,
        name2: &str,
        app2: i64,
        together: i64,
    ) -> DbCoAppearance {
        DbCoAppearance {
            character1_id: id1,
            character1_name: name1.to_string(),
            character1_appearances: app1,
            character2_id: id2,
            character2_name: name2.to_string(),
            character2_appearances: app2,
            comics_together: together,
        }
    }

    #[test]
    fn build_co_appearances_deduplicates_characters() {
        let rows = vec![
            make_co_appearance(1, "Alice", 100, 2, "Bob", 80, 50),
            make_co_appearance(1, "Alice", 100, 3, "Carol", 60, 30),
        ];
        let response = build_co_appearances_response(rows);
        assert_eq!(response.characters.len(), 3);
        assert_eq!(response.characters[&1].name, "Alice");
        assert_eq!(response.characters[&1].appearances, 100);
        assert_eq!(response.pairs.len(), 2);
    }

    #[test]
    fn build_location_affinity_groups_by_location() {
        let rows = vec![
            DbLocationAffinityRow {
                location_id: 10,
                location_name: "Coffee Shop".to_string(),
                character_id: 1,
                character_name: "Alice".to_string(),
                comics_together: 30,
            },
            DbLocationAffinityRow {
                location_id: 10,
                location_name: "Coffee Shop".to_string(),
                character_id: 2,
                character_name: "Bob".to_string(),
                comics_together: 20,
            },
            DbLocationAffinityRow {
                location_id: 20,
                location_name: "Bar".to_string(),
                character_id: 3,
                character_name: "Carol".to_string(),
                comics_together: 15,
            },
        ];
        let result = build_location_affinity_response(rows);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].location_name, "Coffee Shop");
        assert_eq!(result[0].top_characters.len(), 2);
        assert_eq!(result[1].location_name, "Bar");
        assert_eq!(result[1].top_characters.len(), 1);
    }

    #[test]
    fn build_location_affinity_caps_at_five_characters() {
        let rows = (1_u16..=7)
            .map(|i| DbLocationAffinityRow {
                location_id: 10,
                location_name: "Place".to_string(),
                character_id: i,
                character_name: format!("Char{i}"),
                comics_together: i64::from(100 - i),
            })
            .collect();
        let result = build_location_affinity_response(rows);
        assert_eq!(result[0].top_characters.len(), 5);
    }

    #[test]
    fn build_location_spotlight_groups_by_year() {
        let rows = vec![
            DbLocationYearlyAppearanceRow {
                year: Some(2020),
                id: 1,
                name: "Coffee Shop".to_string(),
                color_red: 0,
                color_green: 0,
                color_blue: 0,
                appearances: 50,
            },
            DbLocationYearlyAppearanceRow {
                year: Some(2020),
                id: 2,
                name: "Bar".to_string(),
                color_red: 0,
                color_green: 0,
                color_blue: 0,
                appearances: 30,
            },
            DbLocationYearlyAppearanceRow {
                year: Some(2021),
                id: 1,
                name: "Coffee Shop".to_string(),
                color_red: 0,
                color_green: 0,
                color_blue: 0,
                appearances: 60,
            },
        ];
        let result = build_location_spotlight_response(rows);
        assert_eq!(result.years.len(), 2);
        assert_eq!(result.years[0].locations.len(), 2);
        assert_eq!(result.years[1].locations.len(), 1);
        assert_eq!(result.locations.len(), 2);
    }

    #[test]
    fn build_location_spotlight_caps_at_five() {
        let rows = (1_u16..=7)
            .map(|i| DbLocationYearlyAppearanceRow {
                year: Some(2020),
                id: i,
                name: format!("Loc{i}"),
                color_red: 0,
                color_green: 0,
                color_blue: 0,
                appearances: i64::from(100 - i),
            })
            .collect();
        let result = build_location_spotlight_response(rows);
        assert_eq!(result.years[0].locations.len(), 5);
    }

    #[test]
    fn build_debut_clusters_groups_by_year() {
        let rows = vec![
            DbDebutDetailRow {
                year: Some(2005),
                id: 1,
                name: "Alice".to_string(),
            },
            DbDebutDetailRow {
                year: Some(2005),
                id: 2,
                name: "Bob".to_string(),
            },
            DbDebutDetailRow {
                year: Some(2006),
                id: 3,
                name: "Carol".to_string(),
            },
        ];
        let result = build_debut_clusters_response(rows);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].year, 2005);
        assert_eq!(result[0].characters.len(), 2);
        assert_eq!(result[1].year, 2006);
        assert_eq!(result[1].characters.len(), 1);
    }

    #[test]
    fn build_debut_clusters_skips_null_year() {
        let rows = vec![
            DbDebutDetailRow {
                year: None,
                id: 1,
                name: "Alice".to_string(),
            },
            DbDebutDetailRow {
                year: Some(2005),
                id: 2,
                name: "Bob".to_string(),
            },
        ];
        let result = build_debut_clusters_response(rows);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].year, 2005);
    }

    #[test]
    fn build_location_co_occurrences_deduplicates_locations() {
        let rows = vec![
            DbLocationCoOccurrenceRow {
                location1_id: 1,
                location1_name: "Coffee Shop".to_string(),
                location1_appearances: 100,
                location2_id: 2,
                location2_name: "Bar".to_string(),
                location2_appearances: 80,
                comics_together: 40,
            },
            DbLocationCoOccurrenceRow {
                location1_id: 1,
                location1_name: "Coffee Shop".to_string(),
                location1_appearances: 100,
                location2_id: 3,
                location2_name: "Park".to_string(),
                location2_appearances: 60,
                comics_together: 20,
            },
        ];
        let result = build_location_co_occurrences_response(rows);
        assert_eq!(result.locations.len(), 3);
        assert_eq!(result.pairs.len(), 2);
        assert_eq!(result.locations[&1].name, "Coffee Shop");
    }

    #[test]
    fn build_best_friend_deduplicates_characters() {
        use database::models::stats::CoAppearance as DbCoAppearance;
        let rows = vec![
            DbCoAppearance {
                character1_id: 1,
                character1_name: "Alice".to_string(),
                character1_appearances: 100,
                character2_id: 2,
                character2_name: "Bob".to_string(),
                character2_appearances: 80,
                comics_together: 75,
            },
            DbCoAppearance {
                character1_id: 1,
                character1_name: "Alice".to_string(),
                character1_appearances: 100,
                character2_id: 3,
                character2_name: "Carol".to_string(),
                character2_appearances: 50,
                comics_together: 45,
            },
        ];
        let result = build_best_friend_response(rows);
        assert_eq!(result.characters.len(), 3);
        assert_eq!(result.pairs.len(), 2);
        assert_eq!(result.characters[&1].appearances, 100);
    }
}
