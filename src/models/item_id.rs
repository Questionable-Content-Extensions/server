use serde::{Deserialize, Serialize};

#[derive(
    Copy, Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
#[serde(transparent)]
pub struct ItemId(u16);

derive_transparent_display!(ItemId);

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
