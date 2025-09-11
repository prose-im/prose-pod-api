// prosody-config
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Display, ops::Deref, str::FromStr};

use strum::Display;

use crate::util::split_leading_digits;

// ===== Data =====

/// Bytes.
///
/// See <https://en.wikipedia.org/wiki/Byte#Multiple-byte_units>.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)
)]
pub enum Bytes {
    Bytes(u32),
    KiloBytes(u32),
    KibiBytes(u32),
    MegaBytes(u32),
    MebiBytes(u32),
}

impl FromStr for Bytes {
    type Err = ParseMeasurementError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (n, unit) = split_leading_digits(s);
        let n = u32::from_str(n).map_err(ParseMeasurementError::InvalidQuantity)?;
        match unit {
            "B" => Ok(Self::Bytes(n)),
            "kB" => Ok(Self::KiloBytes(n)),
            "KiB" => Ok(Self::KibiBytes(n)),
            "MB" => Ok(Self::MegaBytes(n)),
            "MiB" => Ok(Self::MebiBytes(n)),
            _ => Err(ParseMeasurementError::InvalidUnit),
        }
    }
}

impl Display for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bytes(n) => write!(f, "{n}B"),
            Self::KiloBytes(n) => write!(f, "{n}kB"),
            Self::KibiBytes(n) => write!(f, "{n}KiB"),
            Self::MegaBytes(n) => write!(f, "{n}MB"),
            Self::MebiBytes(n) => write!(f, "{n}MiB"),
        }
    }
}

/// Data-transfer rate (kB/s, MB/s…).
///
/// See <https://en.wikipedia.org/wiki/Data-rate_units>
/// and <https://docs.ejabberd.im/admin/configuration/basic/#shapers> for ejabberd.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)
)]
pub enum DataRate {
    BytesPerSec(u32),
    KiloBytesPerSec(u32),
    MegaBytesPerSec(u32),
}

impl FromStr for DataRate {
    type Err = ParseMeasurementError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (n, unit) = split_leading_digits(s);
        let n = u32::from_str(n).map_err(ParseMeasurementError::InvalidQuantity)?;
        let unit = unit.replacen("bps", "bit/s", 1).replacen("Bps", "B/s", 1);
        match unit.as_str() {
            "B/s" => Ok(Self::BytesPerSec(n)),
            "kB/s" => Ok(Self::KiloBytesPerSec(n)),
            "MB/s" => Ok(Self::MegaBytesPerSec(n)),
            _ => Err(ParseMeasurementError::InvalidUnit),
        }
    }
}

impl Display for DataRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BytesPerSec(n) => write!(f, "{n}B/s"),
            Self::KiloBytesPerSec(n) => write!(f, "{n}kB/s"),
            Self::MegaBytesPerSec(n) => write!(f, "{n}MB/s"),
        }
    }
}

// ===== Durations =====

#[cfg(not(feature = "serde"))]
pub trait DurationContent {}
#[cfg(feature = "serde")]
pub trait DurationContent: serdev::Serialize + for<'de> serdev::Deserialize<'de> {}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
#[cfg_attr(
    feature = "serde",
    derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)
)]
pub struct Duration<Content: DurationContent>(pub Content);

impl<Content: DurationContent> Deref for Duration<Content> {
    type Target = Content;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Content: DurationContent + FromStr> FromStr for Duration<Content> {
    type Err = <Content as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Content::from_str(s).map(Self)
    }
}

impl<Content: DurationContent + Display> Display for Duration<Content> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)
)]
pub enum TimeLike {
    Seconds(u32),
    Minutes(u32),
    Hours(u32),
}

impl FromStr for TimeLike {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(s) = s.strip_prefix("PT") else {
            return Err("Expected ISO 8601 duration format (missing `PT`).");
        };
        let (n, unit) = split_leading_digits(s);
        let n = u32::from_str(n).map_err(|_| "Invalid quantity.")?;
        match unit {
            "S" => Ok(Self::Seconds(n)),
            "M" => Ok(Self::Minutes(n)),
            "H" => Ok(Self::Hours(n)),
            _ => Err("Invalid unit."),
        }
    }
}

impl Display for TimeLike {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Seconds(n) => write!(f, "PT{n}S"),
            Self::Minutes(n) => write!(f, "PT{n}M"),
            Self::Hours(n) => write!(f, "PT{n}H"),
        }
    }
}

impl TimeLike {
    pub fn seconds(&self) -> u32 {
        match self {
            Self::Seconds(n) => n.clone(),
            Self::Minutes(n) => n * Self::Seconds(60).seconds(),
            Self::Hours(n) => n * Self::Minutes(60).seconds(),
        }
    }
}

impl DurationContent for TimeLike {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)
)]
pub enum DateLike {
    Days(u32),
    Weeks(u32),
    Months(u32),
    Years(u32),
}

impl FromStr for DateLike {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(s) = s.strip_prefix("P") else {
            return Err("Expected ISO 8601 duration format (missing `P`).");
        };
        let (n, unit) = split_leading_digits(s);
        let n = u32::from_str(n).map_err(|_| "Invalid quantity.")?;
        match unit {
            "D" => Ok(Self::Days(n)),
            "W" => Ok(Self::Weeks(n)),
            "M" => Ok(Self::Months(n)),
            "Y" => Ok(Self::Years(n)),
            _ => Err("Invalid unit."),
        }
    }
}

impl Display for DateLike {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Days(n) => write!(f, "P{n}D"),
            Self::Weeks(n) => write!(f, "P{n}W"),
            Self::Months(n) => write!(f, "P{n}M"),
            Self::Years(n) => write!(f, "P{n}Y"),
        }
    }
}

impl DurationContent for DateLike {}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)
)]
pub enum PossiblyInfinite<D> {
    Infinite,
    Finite(D),
}

impl<D: FromStr> FromStr for PossiblyInfinite<D> {
    type Err = <D as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "infinite" => Ok(Self::Infinite),
            s => D::from_str(s).map(Self::Finite),
        }
    }
}

impl<D: Display> Display for PossiblyInfinite<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Infinite => write!(f, "infinite"),
            Self::Finite(d) => Display::fmt(d, f),
        }
    }
}

impl<D> PossiblyInfinite<D> {
    pub fn finite(&self) -> Option<&D> {
        match self {
            Self::Infinite => None,
            Self::Finite(d) => Some(d),
        }
    }
}

impl<D> Into<Option<D>> for PossiblyInfinite<D> {
    fn into(self) -> Option<D> {
        match self {
            Self::Infinite => None,
            Self::Finite(d) => Some(d),
        }
    }
}

// ===== Errors =====

#[derive(Debug, Display)]
pub enum ParseMeasurementError {
    InvalidQuantity(std::num::ParseIntError),
    InvalidUnit,
}
