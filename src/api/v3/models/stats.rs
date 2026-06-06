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
pub struct CoAppearancesResponse {
    pub characters: HashMap<u16, String>,
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
