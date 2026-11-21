use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use anyhow::anyhow;
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s.strip_prefix('#').unwrap_or(s);
        if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(anyhow!(
                "Color must be exactly six hexadecimal digits, optionally prefixed with '#'"
            ));
        }

        // `hex` is six ASCII hex digits, so these slices and parses cannot fail.
        let component = |range| u8::from_str_radix(&hex[range], 16);
        Ok(Self(component(0..2)?, component(2..4)?, component(4..6)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_six_hex_digits_with_or_without_hash() {
        assert_eq!(
            "0a1B2c".parse::<ItemColor>().unwrap(),
            ItemColor(0x0a, 0x1b, 0x2c)
        );
        assert_eq!(
            "#ffFF00".parse::<ItemColor>().unwrap(),
            ItemColor(0xff, 0xff, 0x00)
        );
    }

    #[test]
    fn rejects_trailing_characters() {
        assert!("cccccc0".parse::<ItemColor>().is_err());
        assert!("c".repeat(256).parse::<ItemColor>().is_err());
        assert!("#cccccc ".parse::<ItemColor>().is_err());
    }

    #[test]
    fn rejects_sign_prefixed_components() {
        assert!("+f+f+f".parse::<ItemColor>().is_err());
        assert!("#+f+f+f".parse::<ItemColor>().is_err());
        assert!("-f0000".parse::<ItemColor>().is_err());
    }

    #[test]
    fn rejects_too_short_and_non_hex_input() {
        assert!("".parse::<ItemColor>().is_err());
        assert!("#".parse::<ItemColor>().is_err());
        assert!("fffff".parse::<ItemColor>().is_err());
        assert!("gggggg".parse::<ItemColor>().is_err());
        assert!("##ffffff".parse::<ItemColor>().is_err());
        assert!("ffééff".parse::<ItemColor>().is_err());
    }
}
