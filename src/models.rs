#![allow(clippy::use_self)]

use crate::database::models::Comic as DatabaseComic;
use chrono::{DateTime, Utc};
// {"comic":1,"title":"Employment Sucks","isNonCanon":false,"isGuestComic":false}
use serde::{Deserialize, Serialize};

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
    pub r#type: Option<String>,
    pub color: Option<String>,
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
