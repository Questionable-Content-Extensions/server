use parse_display::Display;
use semval::{context::Context as ValidationContext, Result as ValidationResult, Validate};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

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
pub struct ComicId(u16);

impl ComicId {
    #[inline]
    pub fn into_inner(self) -> u16 {
        self.0
    }
}

impl TryFrom<i16> for ComicId {
    type Error = ();

    #[inline]
    fn try_from(comic_id: i16) -> Result<Self, Self::Error> {
        if comic_id < 1 {
            Err(())
        } else {
            Ok(Self(comic_id as u16))
        }
    }
}

impl Validate for ComicId {
    type Invalidity = ComicIdInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.0 < 1, ComicIdInvalidity::MinValue)
            .into()
    }
}

#[derive(Copy, Clone, Debug, Display, Eq, PartialEq)]
pub enum ComicIdInvalidity {
    #[display("comicId cannot be 0")]
    MinValue,
}
