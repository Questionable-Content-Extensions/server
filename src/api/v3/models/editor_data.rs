use serde::Serialize;
use ts_rs::TS;

use crate::api::v3::models::MissingNavigationData;
use crate::models::{False, True};

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
