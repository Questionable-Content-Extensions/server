use std::collections::HashMap;

use crate::api::v3::models::ItemColor;
use crate::models::{ComicId, ItemId};
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ItemStats {
    pub id: ItemId,
    pub name: String,
    pub first_comic: ComicId,
    pub last_comic: ComicId,
    pub appearances: u32,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CoAppearancePair {
    pub character1_id: ItemId,
    pub character2_id: ItemId,
    pub comics_together: u32,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CoAppearanceCharacterMeta {
    pub name: String,
    pub appearances: u32,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CoAppearancesResponse {
    pub characters: HashMap<u16, CoAppearanceCharacterMeta>,
    pub pairs: Vec<CoAppearancePair>,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CharacterMeta {
    pub name: String,
    #[ts(type = "string")]
    pub color: ItemColor,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct YearlyRankEntry {
    pub id: ItemId,
    pub appearances: u32,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct YearlySpotlightYear {
    pub year: i32,
    pub characters: Vec<YearlyRankEntry>,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct YearlySpotlightResponse {
    pub characters: HashMap<u16, CharacterMeta>,
    pub years: Vec<YearlySpotlightYear>,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DebutsPerYear {
    pub year: i32,
    pub cast_debuts: u32,
    pub location_debuts: u32,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct YearlyOverview {
    pub year: i32,
    pub total_cast: u32,
    pub new_cast: u32,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MonthlyComics {
    pub month: u8,
    pub comics: u32,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DailyComics {
    pub dow: u8,
    pub comics: u32,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PublicationCalendar {
    pub monthly: Vec<MonthlyComics>,
    pub daily: Vec<DailyComics>,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ComebackCharacter {
    pub id: ItemId,
    pub name: String,
    pub last_comic: ComicId,
    pub return_comic: ComicId,
    pub gap_days: u32,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocationAffinityCharacter {
    pub id: ItemId,
    pub name: String,
    pub comics_together: u32,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocationAffinity {
    pub location_id: ItemId,
    pub location_name: String,
    pub top_characters: Vec<LocationAffinityCharacter>,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MostCrowdedComic {
    pub comic_id: ComicId,
    pub cast_count: u32,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AvgCastPerYear {
    pub year: i32,
    pub avg_cast_size: f64,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CrowdedComicsResponse {
    pub top_comics: Vec<MostCrowdedComic>,
    pub avg_per_year: Vec<AvgCastPerYear>,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocationSpotlightYear {
    pub year: i32,
    pub locations: Vec<YearlyRankEntry>,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocationSpotlightResponse {
    pub locations: HashMap<u16, CharacterMeta>,
    pub years: Vec<LocationSpotlightYear>,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PublicationGap {
    pub before_comic: ComicId,
    pub after_comic: ComicId,
    pub gap_days: u32,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DebutCharacter {
    pub id: ItemId,
    pub name: String,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DebutYear {
    pub year: i32,
    pub characters: Vec<DebutCharacter>,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EnsembleRatio {
    pub year: i32,
    pub no_cast: u32,
    pub solo: u32,
    pub small_group: u32,
    pub large_group: u32,
    pub total: u32,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CharacterRegularity {
    pub id: ItemId,
    pub name: String,
    pub appearances: u32,
    pub avg_gap_days: f64,
    pub stddev_gap_days: f64,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocationCoOccurrenceEntry {
    pub id: ItemId,
    pub name: String,
    pub appearances: u32,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocationCoOccurrencePair {
    pub location1_id: ItemId,
    pub location2_id: ItemId,
    pub comics_together: u32,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocationCoOccurrenceResponse {
    pub locations: HashMap<u16, LocationCoOccurrenceEntry>,
    pub pairs: Vec<LocationCoOccurrencePair>,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BestFriendPair {
    pub character1_id: ItemId,
    pub character2_id: ItemId,
    pub comics_together: u32,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BestFriendResponse {
    pub characters: HashMap<u16, CoAppearanceCharacterMeta>,
    pub pairs: Vec<BestFriendPair>,
}
