// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Display, str::FromStr};

/// A color.
///
/// See [color - CSS | MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/color).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(serde_with::DeserializeFromStr, serde_with::SerializeDisplay)]
pub enum Color {
    Hex([u8; 3]),
}

impl FromStr for Color {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("#") {
            if !s.chars().skip(1).all(|c| c.is_ascii_hexdigit()) {
                return Err("Invalid Hex string: Bad chars.");
            }
            let s = s.as_bytes();
            let rgb = match s.len() {
                4 => [
                    hex_to_u8(s[1], s[1]),
                    hex_to_u8(s[2], s[2]),
                    hex_to_u8(s[3], s[3]),
                ],
                7 => [
                    hex_to_u8(s[1], s[2]),
                    hex_to_u8(s[3], s[4]),
                    hex_to_u8(s[5], s[6]),
                ],
                _ => return Err("Invalid Hex string: Wrong length."),
            };
            Ok(Self::Hex(rgb))
        } else {
            Err("Not supported yet.")
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Hex([r, g, b]) => f.write_str(&format!("#{r:02x}{g:02x}{b:02x}")),
        }
    }
}

fn hex_to_u8(h: u8, l: u8) -> u8 {
    fn to_u8(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            _ => panic!("invalid hex"),
        }
    }
    (to_u8(h) << 4) | to_u8(l)
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::Color;

    #[test]
    fn test_from_hex() {
        // 6 digits
        assert_eq!(Color::from_str("#000000"), Ok(Color::Hex([0, 0, 0])));
        assert_eq!(Color::from_str("#FFFFFF"), Ok(Color::Hex([255, 255, 255])));
        assert_eq!(Color::from_str("#ffffff"), Ok(Color::Hex([255, 255, 255])));
        assert_eq!(Color::from_str("#012345"), Ok(Color::Hex([1, 35, 69])));

        // 3 digits
        assert_eq!(Color::from_str("#000"), Color::from_str("#000000"));
        assert_eq!(Color::from_str("#FFF"), Color::from_str("#FFFFFF"));
        assert_eq!(Color::from_str("#fff"), Color::from_str("#ffffff"));
        assert_eq!(Color::from_str("#123"), Ok(Color::Hex([17, 34, 51])));

        // Invalid
        assert_eq!(Color::from_str("#f").ok(), None);
        assert_eq!(Color::from_str("#gggggg").ok(), None);
    }

    #[test]
    fn test_hex_to_string() {
        assert_eq!(Color::Hex([0, 0, 0]).to_string().as_str(), "#000000");
        assert_eq!(Color::Hex([255, 255, 255]).to_string().as_str(), "#ffffff");
        assert_eq!(Color::Hex([1, 35, 69]).to_string().as_str(), "#012345");
        assert_eq!(Color::Hex([17, 34, 51]).to_string().as_str(), "#112233");
    }

    #[test]
    fn test_unsupported() {
        assert_eq!(Color::from_str("red").ok(), None);
        assert_eq!(Color::from_str("hwb(152deg 0% 58% / 70%)").ok(), None);
    }
}
