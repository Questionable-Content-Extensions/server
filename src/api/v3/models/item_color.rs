use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use anyhow::{anyhow, Context};
use serde::{Serialize, Serializer};

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct ItemColor(pub u8, pub u8, pub u8);

impl Display for ItemColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if f.alternate() {
            Ok(write!(f, "#{:02x}{:02x}{:02x}", self.0, self.1, self.2)?)
        } else {
            Ok(write!(f, "{:02x}{:02x}{:02x}", self.0, self.1, self.2)?)
        }
    }
}

impl Serialize for ItemColor {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl FromStr for ItemColor {
    type Err = anyhow::Error;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        if let Some(sp) = s.strip_prefix('#') {
            s = sp;
        }

        let red = s
            .get(0..2)
            .ok_or_else(|| anyhow!("Color is missing red component value"))
            .and_then(|s| {
                u8::from_str_radix(s, 16)
                    .context("Red component of color is not a valid hexadecimal value")
            })?;
        let green = s
            .get(2..4)
            .ok_or_else(|| anyhow!("Color is missing green component value"))
            .and_then(|s| {
                u8::from_str_radix(s, 16)
                    .context("Green component of color is not a valid hexadecimal value")
            })?;
        let blue = s
            .get(4..6)
            .ok_or_else(|| anyhow!("Color is missing blue component value"))
            .and_then(|s| {
                u8::from_str_radix(s, 16)
                    .context("Blue component of color is not a valid hexadecimal value")
            })?;

        Ok(Self(red, green, blue))
    }
}
