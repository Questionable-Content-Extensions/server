use database::models::ComicId as DatabaseComicId;
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
pub struct ComicId(u16);

impl ComicId {
    #[inline]
    pub fn into_inner(self) -> u16 {
        self.0
    }
}

impl From<u16> for ComicId {
    fn from(comic_id: u16) -> Self {
        Self(comic_id)
    }
}

impl From<DatabaseComicId> for ComicId {
    fn from(comic_id: DatabaseComicId) -> Self {
        Self(comic_id.into_inner())
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

    fn validate(&self) -> semval::ValidationResult<Self::Invalidity> {
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
