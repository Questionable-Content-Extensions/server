use parse_display::Display;
use semval::{context::Context as ValidationContext, Validate};
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
pub struct ItemId(u16);

impl ItemId {
    #[inline]
    pub fn into_inner(self) -> u16 {
        self.0
    }
}

impl From<u16> for ItemId {
    #[inline]
    fn from(comic_id: u16) -> Self {
        Self(comic_id)
    }
}

impl Validate for ItemId {
    type Invalidity = ItemIdInvalidity;

    fn validate(&self) -> semval::ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.0 < 1, ItemIdInvalidity::MinValue)
            .into()
    }
}

#[derive(Copy, Clone, Debug, Display, Eq, PartialEq)]
pub enum ItemIdInvalidity {
    #[display("itemId cannot be 0")]
    MinValue,
}
