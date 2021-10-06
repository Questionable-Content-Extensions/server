use serde::{Deserialize, Serialize};

#[derive(
    Copy, Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
#[serde(transparent)]
pub struct ComicId(u16);

derive_transparent_display!(ComicId);

impl ComicId {
    #[inline]
    pub fn into_inner(self) -> u16 {
        self.0
    }
}

impl From<i16> for ComicId {
    #[inline]
    fn from(comic_id: i16) -> Self {
        Self(comic_id as u16)
    }
}
