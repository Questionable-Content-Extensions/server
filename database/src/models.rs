mod comic;
mod item;
mod item_type;
mod log_entry;
mod news;
mod occurrence;
mod token;

use std::borrow::Borrow;

pub use comic::*;
pub use item::*;
pub use item_type::*;
pub use log_entry::*;
pub use news::*;
pub use occurrence::*;
pub use token::*;

#[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ComicId(u16);

impl ComicId {
    #[inline]
    pub fn as_inner(&self) -> &u16 {
        &self.0
    }

    #[inline]
    pub fn into_inner(self) -> u16 {
        self.0
    }
}

impl PartialEq<u16> for ComicId {
    fn eq(&self, other: &u16) -> bool {
        self.0.eq(other)
    }
}

impl From<u16> for ComicId {
    fn from(comic_id: u16) -> Self {
        Self(comic_id)
    }
}

impl Borrow<u16> for ComicId {
    fn borrow(&self) -> &u16 {
        self.as_inner()
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ItemId(u16);

impl ItemId {
    #[inline]
    pub fn as_inner(&self) -> &u16 {
        &self.0
    }

    #[inline]
    pub fn into_inner(self) -> u16 {
        self.0
    }
}

impl PartialEq<u16> for ItemId {
    fn eq(&self, other: &u16) -> bool {
        self.0.eq(other)
    }
}

impl From<u16> for ItemId {
    fn from(item_id: u16) -> Self {
        Self(item_id)
    }
}

impl Borrow<u16> for ItemId {
    fn borrow(&self) -> &u16 {
        self.as_inner()
    }
}
