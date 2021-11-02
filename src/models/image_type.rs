use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[repr(i32)]
pub enum ImageType {
    Unknown = 0,
    Png = 1,
    Gif = 2,
    Jpeg = 3,
}

impl From<i32> for ImageType {
    #[inline]
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
