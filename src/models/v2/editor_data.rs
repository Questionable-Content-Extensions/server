use serde::Serialize;
use ts_rs::TS;

use crate::models::v1;
use crate::models::v2::{False, MissingNavigationData, True};

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
