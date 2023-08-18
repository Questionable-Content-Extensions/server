use anyhow::bail;
use database::models::ItemType as DatabaseItemType;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use ts_rs::TS;

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ItemType {
    Cast,
    Location,
    Storyline,
}

impl ItemType {
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cast => "cast",
            Self::Location => "location",
            Self::Storyline => "storyline",
        }
    }
}

impl TryFrom<&'_ str> for ItemType {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(item_type: &'_ str) -> Result<Self, Self::Error> {
        Ok(match item_type {
            "cast" => Self::Cast,
            "location" => Self::Location,
            "storyline" => Self::Storyline,
            _ => bail!("Invalid item type value: {}", item_type),
        })
    }
}

impl From<ItemType> for DatabaseItemType {
    fn from(i: ItemType) -> Self {
        match i {
            ItemType::Cast => DatabaseItemType::Cast,
            ItemType::Location => DatabaseItemType::Location,
            ItemType::Storyline => DatabaseItemType::Storyline,
        }
    }
}
