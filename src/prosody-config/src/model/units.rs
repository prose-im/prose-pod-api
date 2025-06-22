// prosody-config
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Display, num::ParseIntError, ops::Deref, str::FromStr};

use strum::Display;

use crate::util::split_leading_digits;

// ===== Data =====

/// Bytes.
///
/// See <https://en.wikipedia.org/wiki/Byte#Multiple-byte_units>.
#[derive(Debug, Clone, Eq, PartialEq)]
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
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DataRate {
    BytesPerSec(u32),
    KiloBytesPerSec(u32),
    MegaBytesPerSec(u32),
}

// ===== Durations =====

pub trait DurationContent {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Duration<Content: DurationContent>(pub Content);

impl<Content: DurationContent> Deref for Duration<Content> {
    type Target = Content;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TimeLike {
    Seconds(u32),
    Minutes(u32),
    Hours(u32),
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DateLike {
    Days(u32),
    Weeks(u32),
    Months(u32),
    Years(u32),
}

impl DurationContent for DateLike {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PossiblyInfinite<D> {
    Infinite,
    Finite(D),
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
    InvalidQuantity(ParseIntError),
    InvalidUnit,
}
