use std::collections::HashMap;

use crate::api::v3::models::ItemColor;
use crate::api::v3::models::stats::{
    CharacterMeta, CoAppearanceCharacterMeta, CoAppearancePair, CoAppearancesResponse,
    ComebackCharacter, DailyComics, DebutsPerYear, ItemStats, LocationAffinity,
    LocationAffinityCharacter, MonthlyComics, PublicationCalendar, YearlyOverview, YearlyRankEntry,
    YearlySpotlightResponse, YearlySpotlightYear,
};
use crate::models::{ComicId, ItemId};
use actix_web::{HttpResponse, Result, error, web};
use database::DbPool;
use database::models::stats::{
    CoAppearance as DbCoAppearance, ComebackCharacterRow as DbComebackCharacterRow,
    DebutsPerYearRow as DbDebutsPerYearRow, ItemStats as DbItemStats,
    LocationAffinityRow as DbLocationAffinityRow, PublicationDowRow as DbPublicationDowRow,
    PublicationMonthRow as DbPublicationMonthRow, YearlyAppearanceRow as DbYearlyAppearanceRow,
    YearlyOverviewRow as DbYearlyOverviewRow,
};
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
        .service(web::resource("/location-affinity").route(web::get().to(location_affinity)));
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
}
