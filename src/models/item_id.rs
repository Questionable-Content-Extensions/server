use parse_display::Display;
use semval::context::Context as ValidationContext;
use semval::{Result as ValidationResult, Validate};
use serde::{Deserialize, Serialize};

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
)]
#[serde(transparent)]
pub struct ItemId(u16);

impl ItemId {
    #[inline]
    pub fn into_inner(self) -> u16 {
        self.0
    }
}

impl From<i16> for ItemId {
    #[inline]
    fn from(comic_id: i16) -> Self {
        Self(comic_id as u16)
    }
}

impl Validate for ItemId {
    type Invalidity = ItemIdInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.0 < 1, ItemIdInvalidity::MinValue)
            .into()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ItemIdInvalidity {
    MinValue,
}
