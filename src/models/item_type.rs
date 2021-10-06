use anyhow::bail;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::convert::TryFrom;

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
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
