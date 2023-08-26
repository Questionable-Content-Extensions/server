use parse_display::Display;
use semval::context::Context as ValidationContext;
use semval::Validate;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Deserialize,
    Display,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    TS,
)]
#[serde(transparent)]
#[ts(export)]
pub struct ImageId(u32);

impl ImageId {
    #[inline]
    pub fn into_inner(self) -> u32 {
        self.0
    }
}

impl From<u32> for ImageId {
    #[inline]
    fn from(image_id: u32) -> Self {
        Self(image_id)
    }
}

impl Validate for ImageId {
    type Invalidity = ImageIdInvalidity;

    fn validate(&self) -> semval::ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.0 < 1, ImageIdInvalidity::MinValue)
            .into()
    }
}

#[derive(Copy, Clone, Debug, Display, Eq, PartialEq)]
pub enum ImageIdInvalidity {
    #[display("imageId cannot be 0")]
    MinValue,
}
