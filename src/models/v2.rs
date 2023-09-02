use chrono::{DateTime, Utc};
use database::models::{Comic as DatabaseComic, ItemImageMetadata};
use serde::Serialize;
use ts_rs::TS;

pub use super::v1::*;

mod editor_data;
pub use editor_data::{EditorData, MissingEditorData};

mod image_id;
pub use image_id::{ImageId, ImageIdInvalidity};

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Comic {
    pub comic: ComicId,
    pub editor_data: EditorData,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[ts(optional)]
    pub all_items: Vec<ItemNavigationData>,
    #[serde(flatten)]
    pub data: ComicData,
}

#[derive(Debug, Serialize, TS)]
#[serde(untagged)]
#[ts(export)]
#[allow(variant_size_differences)]
pub enum ComicData {
    Missing(MissingComic),
    Present(PresentComic),
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PresentComic {
    pub has_data: True,
    pub image_type: Option<ImageType>,
    #[ts(type = "string | null")]
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

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Item {
    pub id: ItemId,
    pub short_name: String,
    pub name: String,
    pub r#type: ItemType,
    #[ts(type = "string")]
    pub color: ItemColor,
    pub first: ComicId,
    pub last: ComicId,
    pub appearances: i32,
    pub total_comics: i32,
    pub presence: f64,
    pub has_image: bool,
    pub primary_image: Option<u32>,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ItemImageList {
    pub id: ImageId,
    // Needed because ts-rs transforms the name to `crc32CHash`, which
    // differs from what serde does.
    // <https://github.com/Aleph-Alpha/ts-rs/issues/165>
    #[ts(rename = "crc32cHash")]
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

// TODO: Update RelatedItem to match new slimmer ItemNavigationData

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
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

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
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

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RelatedItem {
    pub id: ItemId,
    pub count: i32,
}
