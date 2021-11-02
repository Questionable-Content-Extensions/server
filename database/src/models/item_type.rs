use std::convert::TryFrom;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, sqlx::Type)]
#[sqlx(rename_all = "camelCase")]
pub enum ItemType {
    Cast,
    Location,
    Storyline,
}

impl ItemType {
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cast => "cast",
            Self::Location => "location",
            Self::Storyline => "storyline",
        }
    }
}

impl<'a> TryFrom<&'a str> for ItemType {
    type Error = &'a str;

    #[inline]
    fn try_from(item_type: &'a str) -> Result<Self, Self::Error> {
        Ok(match item_type {
            "cast" => Self::Cast,
            "location" => Self::Location,
            "storyline" => Self::Storyline,
            _ => return Err(item_type),
        })
    }
}
