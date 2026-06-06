use std::collections::HashMap;

use crate::api::v3::models::ItemColor;
use crate::api::v3::models::stats::{
    CharacterMeta, CoAppearancePair, CoAppearancesResponse, ItemStats, YearlyRankEntry,
    YearlySpotlightResponse, YearlySpotlightYear,
};
use crate::models::{ComicId, ItemId};
use actix_web::{HttpResponse, Result, error, web};
use database::DbPool;
use database::models::stats::{
    CoAppearance as DbCoAppearance, ItemStats as DbItemStats,
    YearlyAppearanceRow as DbYearlyAppearanceRow,
};
use tracing::{Instrument, info_span};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/cast").route(web::get().to(cast)))
        .service(web::resource("/locations").route(web::get().to(locations)))
        .service(web::resource("/co-appearances").route(web::get().to(co_appearances)))
        .service(web::resource("/yearly-spotlight").route(web::get().to(yearly_spotlight)));
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

fn item_stats_from_db(row: DbItemStats) -> ItemStats {
    ItemStats {
        id: ItemId::from(row.id),
        name: row.name,
        first_comic: ComicId::from(row.first_comic.unwrap_or(0)),
        last_comic: ComicId::from(row.last_comic.unwrap_or(0)),
        appearances: u32::try_from(row.appearances).unwrap_or(u32::MAX),
    }
}

fn build_co_appearances_response(rows: Vec<DbCoAppearance>) -> CoAppearancesResponse {
    let mut characters: HashMap<u16, String> = HashMap::new();
    let mut pairs = Vec::with_capacity(rows.len());

    for row in rows {
        characters
            .entry(row.character1_id)
            .or_insert(row.character1_name);
        characters
            .entry(row.character2_id)
            .or_insert(row.character2_name);
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
