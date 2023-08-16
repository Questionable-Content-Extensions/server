#![allow(clippy::use_self)]

use chrono::TimeZone;
use chrono::{DateTime, Utc};
use database::models::{Comic as DatabaseComic, ItemImageMetadata, LogListEntry};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

mod comic_id;
mod image_type;
mod item_color;
mod item_id;
mod item_type;
mod set_boolean;
mod token;

pub use comic_id::*;
pub use image_type::ImageType;
pub use item_color::ItemColor;
pub use item_id::*;
pub use item_type::ItemType;
pub use set_boolean::*;
pub use token::Token;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComicList {
    pub comic: ComicId,
    pub title: String,
    pub is_non_canon: bool,
    pub is_guest_comic: bool,
}

impl From<DatabaseComic> for ComicList {
    fn from(c: DatabaseComic) -> Self {
        Self {
            comic: ComicId::try_from(c.id).expect("database has valid comicIds"),
            title: c.title,
            is_guest_comic: c.is_guest_comic != 0,
            is_non_canon: c.is_non_canon != 0,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comic {
    pub comic: ComicId,
    pub editor_data: EditorData,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub all_items: Vec<ItemNavigationData>,
    #[serde(flatten)]
    pub data: ComicData,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
#[allow(variant_size_differences)]
pub enum ComicData {
    Missing(MissingComic),
    Present(PresentComic),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissingComic {
    pub has_data: False,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentComic {
    pub image_type: Option<ImageType>,
    pub has_data: True,
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
    pub previous: Option<ComicId>,
    pub next: Option<ComicId>,
    pub items: Vec<ItemNavigationData>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemList {
    pub id: ItemId,
    pub short_name: String,
    pub name: String,
    pub r#type: ItemType,
    pub color: ItemColor,
    pub count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: ItemId,
    pub short_name: String,
    pub name: String,
    pub r#type: ItemType,
    pub color: ItemColor,
    pub first: ComicId,
    pub last: ComicId,
    pub appearances: i64,
    pub total_comics: i64,
    pub presence: f64,
    pub has_image: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub identifier: String,
    pub date_time: DateTime<Utc>,
    pub action: String,
}

impl From<LogListEntry> for LogEntry {
    fn from(l: LogListEntry) -> Self {
        Self {
            identifier: l.identifier,
            date_time: Utc.from_utc_datetime(&l.date_time),
            action: l.action,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedItem {
    pub id: ItemId,
    pub short_name: String,
    pub name: String,
    pub r#type: ItemType,
    pub color: ItemColor,
    pub count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemImageList {
    pub id: u32,
    pub crc32c_hash: u32,
}

impl From<ItemImageMetadata> for ItemImageList {
    #[inline]
    fn from(ii: ItemImageMetadata) -> Self {
        Self {
            id: ii.id,
            crc32c_hash: ii.crc32c_hash,
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct MissingEditorData {
    pub present: False,
}

#[derive(Debug, Serialize)]
pub struct PresentEditorData {
    pub present: True,
    pub missing: MissingNavigationData,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
#[allow(variant_size_differences)]
pub enum EditorData {
    Missing(MissingEditorData),
    Present(PresentEditorData),
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
    pub first: Option<ComicId>,
    pub previous: Option<ComicId>,
    pub next: Option<ComicId>,
    pub last: Option<ComicId>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemNavigationData {
    pub id: ItemId,
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
