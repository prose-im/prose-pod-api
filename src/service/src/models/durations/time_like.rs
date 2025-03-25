// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use iso8601_duration::Duration as ISODuration;

use super::DurationContent;

#[derive(Copy, Clone, Debug, PartialEq)]
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
    /// NOTE: This method is not correct, as a `u32` can overflow in a `f32`.
    ///   As this situation will probably never happen, it's good enough.
    pub fn into_iso_duration(self) -> ISODuration {
        match self {
            Self::Seconds(n) => ISODuration::new(0., 0., 0., 0., 0., n as f32),
            Self::Minutes(n) => ISODuration::new(0., 0., 0., 0., n as f32, 0.),
            Self::Hours(n) => ISODuration::new(0., 0., 0., n as f32, 0., 0.),
        }
    }
    /// NOTE: This method is not correct, as a `u32` can overflow in a `f32`.
    ///   As this situation will probably never happen, it's good enough.
    pub fn into_std_duration(self) -> std::time::Duration {
        match self {
            Self::Seconds(n) => std::time::Duration::from_secs(n as u64),
            Self::Minutes(n) => std::time::Duration::from_secs(60 * n as u64),
            Self::Hours(n) => std::time::Duration::from_secs(3600 * n as u64),
        }
    }
}

impl Eq for TimeLike {}
impl Into<ISODuration> for TimeLike {
    fn into(self) -> ISODuration {
        self.into_iso_duration()
    }
}
impl Into<std::time::Duration> for TimeLike {
    fn into(self) -> std::time::Duration {
        self.into_std_duration()
    }
}
impl TryFrom<ISODuration> for TimeLike {
    type Error = &'static str;

    /// NOTE: This method is not correct, as a `u32` can overflow in a `f32`.
    ///   As this situation will probably never happen, it's good enough.
    fn try_from(value: ISODuration) -> Result<Self, Self::Error> {
        fn non_zero(n: f32) -> Option<u32> {
            match n as u32 {
                0 => None,
                n => Some(n),
            }
        }

        if let Some(hours) = value.num_hours().and_then(non_zero) {
            Ok(Self::Hours(hours))
        } else if let Some(minutes) = value.num_minutes().and_then(non_zero) {
            Ok(Self::Minutes(minutes))
        } else if let Some(seconds) = value.num_seconds().and_then(non_zero) {
            Ok(Self::Seconds(seconds))
        } else {
            Err("Invalid duration")
        }
    }
}
impl DurationContent for TimeLike {
    fn type_name() -> String {
        "Time".to_owned()
    }
}
impl Display for TimeLike {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.into_iso_duration(), f)
    }
}
impl FromStr for TimeLike {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let iso_duration = ISODuration::parse(s).map_err(|e| format!("Parse error: {e:?}"))?;
        Self::try_from(iso_duration).map_err(ToString::to_string)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::models::durations::{Duration, PossiblyInfinite};

    #[test]
    fn test_deserializing() -> Result<(), serde_json::Error> {
        fn test(
            str: &str,
            expected: PossiblyInfinite<Duration<TimeLike>>,
        ) -> Result<(), serde_json::Error> {
            let value = json!(str);
            let duration: PossiblyInfinite<Duration<TimeLike>> = serde_json::from_value(value)?;
            assert_eq!(duration, expected, "{str}");
            Ok(())
        }
        test("infinite", PossiblyInfinite::Infinite)?;
        test(
            "PT2S",
            PossiblyInfinite::Finite(Duration(TimeLike::Seconds(2))),
        )?;
        test(
            "PT3M",
            PossiblyInfinite::Finite(Duration(TimeLike::Minutes(3))),
        )?;
        test(
            "PT4H",
            PossiblyInfinite::Finite(Duration(TimeLike::Hours(4))),
        )?;
        Ok(())
    }
}
