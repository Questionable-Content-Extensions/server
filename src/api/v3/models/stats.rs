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
