use serde::Serialize;
use ts_rs::TS;

pub use super::v1::*;

mod editor_data;
pub use editor_data::{EditorData, MissingEditorData};

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
