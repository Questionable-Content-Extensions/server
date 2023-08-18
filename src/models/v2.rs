#![allow(clippy::use_self)]

use serde::Serialize;
use ts_rs::TS;

use super::v1;
pub use super::v1::*;

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
// Can't export this one; `#[serde(flatten)]` is not supported by ts-rs on `ComicData`
// <https://github.com/Aleph-Alpha/ts-rs/issues/96>
//#[ts(export)]
pub struct Comic {
    pub comic: ComicId,
    pub editor_data: EditorData,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub all_items: Vec<ItemNavigationData>,
    #[serde(flatten)]
    pub data: ComicData,
}

#[derive(Debug, Serialize, TS)]
#[serde(untagged)]
#[ts(export)]
#[allow(variant_size_differences)]
pub enum EditorData {
    Missing(MissingEditorData),
    Present(PresentEditorData),
}

#[derive(Debug, Default, Serialize, TS)]
#[ts(export)]
pub struct MissingEditorData {
    pub present: False,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct PresentEditorData {
    pub present: True,
    pub missing: MissingNavigationData,
}
impl From<v1::EditorData> for PresentEditorData {
    fn from(editor_data: v1::EditorData) -> Self {
        Self {
            present: True::default(),
            missing: editor_data.missing,
        }
    }
}
