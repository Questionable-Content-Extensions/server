use serde::Serialize;

use crate::api::v2::models::MissingNavigationData;
use crate::models::{False, True};

#[derive(Debug, Serialize)]
#[serde(untagged)]
#[allow(variant_size_differences)]
pub enum EditorData {
    Missing(MissingEditorData),
    Present(PresentEditorData),
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
