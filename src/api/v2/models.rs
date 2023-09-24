use crate::models::{ComicId, False, ImageId, ItemId, True};
use chrono::{DateTime, TimeZone, Utc};
use database::models::{Comic as DatabaseComic, ItemImageMetadata, LogListEntry};
use serde::{Deserialize, Serialize};

mod editor_data;
pub use editor_data::*;

mod image_type;
pub use image_type::*;

mod item_color;
pub use item_color::*;

mod item_type;
pub use item_type::*;

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
    pub has_data: True,
    pub image_type: Option<ImageType>,
    pub publish_date: Option<DateTime<Utc>>,
    pub is_accurate_publish_date: bool,
    pub title: String,
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
    pub count: i32,
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
    pub appearances: i32,
    pub total_comics: i32,
    pub presence: f64,
    pub has_image: bool,
    pub primary_image: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemImageList {
    pub id: ImageId,
    // Needed because ts-rs transforms the name to `crc32CHash`, which
    // differs from what serde does.
    // <https://github.com/Aleph-Alpha/ts-rs/issues/165>
    pub crc32c_hash: u32,
}

impl From<ItemImageMetadata> for ItemImageList {
    #[inline]
    fn from(ii: ItemImageMetadata) -> Self {
        Self {
            id: ii.id.into(),
            crc32c_hash: ii.crc32c_hash,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct NavigationData {
    pub first: Option<ComicId>,
    pub previous: Option<ComicId>,
    pub next: Option<ComicId>,
    pub last: Option<ComicId>,
}

#[derive(Debug, Serialize)]
pub struct MissingNavigationData {
    pub cast: NavigationData,
    pub location: NavigationData,
    pub storyline: NavigationData,
    pub title: NavigationData,
    pub tagline: NavigationData,
}

#[derive(Debug)]
pub struct UnhydratedItemNavigationData {
    pub id: ItemId,
    pub navigation_data: NavigationData,
    pub count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemNavigationData {
    pub id: ItemId,
    #[serde(flatten)]
    pub navigation_data: NavigationData,
}
impl From<UnhydratedItemNavigationData> for ItemNavigationData {
    #[inline]
    fn from(unhydrated: UnhydratedItemNavigationData) -> Self {
        let UnhydratedItemNavigationData {
            id,
            navigation_data,
            count: _,
        } = unhydrated;
        Self {
            id,
            navigation_data,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComicList {
    pub comic: ComicId,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tagline: Option<String>,
    pub is_non_canon: bool,
    pub is_guest_comic: bool,
}

impl From<DatabaseComic> for ComicList {
    fn from(c: DatabaseComic) -> Self {
        Self {
            comic: ComicId::try_from(c.id).expect("database has valid comicIds"),
            title: c.title,
            tagline: c.tagline,
            is_guest_comic: c.is_guest_comic != 0,
            is_non_canon: c.is_non_canon != 0,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedItem {
    pub id: ItemId,
    pub count: i32,
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
