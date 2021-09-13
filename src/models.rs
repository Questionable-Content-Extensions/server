#![allow(clippy::use_self)]

use crate::database::models::Comic as DatabaseComic;
use anyhow::bail;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};
use std::convert::TryFrom;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComicList {
    pub comic: i16,
    pub title: String,
    pub is_non_canon: bool,
    pub is_guest_comic: bool,
}

impl From<DatabaseComic> for ComicList {
    fn from(c: DatabaseComic) -> Self {
        Self {
            comic: c.id,
            title: c.title,
            is_guest_comic: c.isGuestComic != 0,
            is_non_canon: c.isNonCanon != 0,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comic {
    pub comic: i16,
    pub image_type: Option<ImageType>,
    pub has_data: bool,
    pub publish_date: Option<DateTime<Utc>>,
    pub is_accurate_publish_date: bool,
    pub title: Option<String>,
    pub tagline: Option<String>,
    pub is_guest_comic: bool,
    pub is_non_canon: bool,
    pub has_no_cast: bool,
    pub has_no_location: bool,
    pub has_no_storyline: bool,
    pub has_no_title: bool,
    pub has_no_tagline: bool,
    pub news: Option<String>,
    pub previous: Option<i16>,
    pub next: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_data: Option<EditorData>,
    pub items: Vec<ItemNavigationData>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub all_items: Vec<ItemNavigationData>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemList {
    pub id: i16,
    pub short_name: String,
    pub name: String,
    pub r#type: ItemType,
    pub color: ItemColor,
    pub count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: i16,
    pub short_name: String,
    pub name: String,
    pub r#type: ItemType,
    pub color: ItemColor,
    pub first: i16,
    pub last: i16,
    pub appearances: i64,
    pub total_comics: i64,
    pub presence: f64,
    pub has_image: bool,
}

#[derive(Debug)]
pub struct ItemColor(u8, u8, u8);
impl ItemColor {
    #[inline]
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self(red, green, blue)
    }
}
impl Serialize for ItemColor {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let color = format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2);
        serializer.serialize_str(&color)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedItem {
    pub id: i16,
    pub short_name: String,
    pub name: String,
    pub r#type: ItemType,
    pub color: ItemColor,
    pub count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemImageList {
    pub id: i32,
    pub crc32c_hash: u32,
}

#[derive(Debug, Serialize)]
pub struct EditorData {
    pub missing: MissingNavigationData,
}

#[derive(Debug, Serialize)]
pub struct MissingNavigationData {
    pub cast: NavigationData,
    pub location: NavigationData,
    pub storyline: NavigationData,
    pub title: NavigationData,
    pub tagline: NavigationData,
}

#[derive(Debug, Serialize)]
pub struct NavigationData {
    pub first: Option<i16>,
    pub previous: Option<i16>,
    pub next: Option<i16>,
    pub last: Option<i16>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemNavigationData {
    pub id: i16,
    pub short_name: Option<String>,
    pub name: Option<String>,
    pub r#type: Option<ItemType>,
    pub color: Option<ItemColor>,
    #[serde(flatten)]
    pub navigation_data: NavigationData,
    pub count: i64,
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Exclusion {
    Guest,
    NonCanon,
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Inclusion {
    All,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[repr(i32)]
pub enum ImageType {
    Unknown = 0,
    Png = 1,
    Gif = 2,
    Jpeg = 3,
}

impl From<i32> for ImageType {
    #[inline]
    fn from(image_type: i32) -> Self {
        match image_type {
            0 => Self::Unknown,
            1 => Self::Png,
            2 => Self::Gif,
            3 => Self::Jpeg,
            _ => unreachable!("Invalid image type value: {}", image_type),
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, sqlx::Type)]
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

pub mod token_permissions {
    pub const HAS_VALID_TOKEN: &str = "HAS_VALID_TOKEN";
    pub const CAN_ADD_ITEM_TO_COMIC: &str = "CAN_ADD_ITEM_TO_COMIC";
    pub const CAN_REMOVE_ITEM_FROM_COMIC: &str = "CAN_REMOVE_ITEM_FROM_COMIC";
    pub const CAN_CHANGE_COMIC_DATA: &str = "CAN_CHANGE_COMIC_DATA";
    pub const CAN_ADD_IMAGE_TO_ITEM: &str = "CAN_ADD_IMAGE_TO_ITEM";
    pub const CAN_REMOVE_IMAGE_FROM_ITEM: &str = "CAN_REMOVE_IMAGE_FROM_ITEM";
    pub const CAN_CHANGE_ITEM_DATA: &str = "CAN_CHANGE_ITEM_DATA";
}
