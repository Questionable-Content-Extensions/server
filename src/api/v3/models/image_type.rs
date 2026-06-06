use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[repr(i32)]
pub enum ImageType {
    Unknown = 0,
    Png = 1,
    Gif = 2,
    Jpeg = 3,
}

impl ImageType {
    pub fn from_trusted(trusted_image_type: i32) -> Self {
        match trusted_image_type {
            0 => Self::Unknown,
            1 => Self::Png,
            2 => Self::Gif,
            3 => Self::Jpeg,
            _ => unreachable!("Invalid image type value: {trusted_image_type}"),
        }
    }
}
