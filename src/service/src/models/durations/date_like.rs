// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use iso8601_duration::Duration as ISODuration;
use time::Duration;

use super::DurationContent;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DateLike {
    Days(u32),
    Weeks(u32),
    Months(u32),
    Years(u32),
}

impl DateLike {
    /// NOTE: This method is not correct, as a `u32` can overflow in a `f32`.
    ///   As this situation will probably never happen, it's good enough.
    pub fn into_iso_duration(self) -> ISODuration {
        match self {
            Self::Days(n) => ISODuration::new(0., 0., n as f32, 0., 0., 0.),
            Self::Weeks(n) => ISODuration::new(0., 0., (n as f32) * 7., 0., 0., 0.),
            Self::Months(n) => ISODuration::new(0., n as f32, 0., 0., 0., 0.),
            Self::Years(n) => ISODuration::new(n as f32, 0., 0., 0., 0., 0.),
        }
    }
    pub fn into_time_delta(self) -> Duration {
        match self {
            Self::Days(n) => Duration::days(n as i64),
            Self::Weeks(n) => Duration::weeks(n as i64),
            Self::Months(n) => Duration::days(30 * n as i64),
            Self::Years(n) => Duration::days(365 * n as i64),
        }
    }
}
impl Eq for DateLike {}
impl Into<ISODuration> for DateLike {
    fn into(self) -> ISODuration {
        self.into_iso_duration()
    }
}
impl TryFrom<ISODuration> for DateLike {
    type Error = &'static str;

    /// NOTE: This method is not correct, as a `f32` can overflow in a `u32`.
    ///   As this situation will probably never happen, it's good enough.
    fn try_from(value: ISODuration) -> Result<Self, Self::Error> {
        fn non_zero(n: f32) -> Option<u32> {
            match n as u32 {
                0 => None,
                n => Some(n),
            }
        }

        if let Some(years) = value.num_years().and_then(non_zero) {
            Ok(Self::Years(years))
        } else if let Some(months) = value.num_months().and_then(non_zero) {
            Ok(Self::Months(months))
        } else if let Some(weeks) = value.num_weeks().and_then(non_zero) {
            Ok(Self::Weeks(weeks))
        } else if let Some(days) = value.num_days().and_then(non_zero) {
            Ok(Self::Days(days))
        } else {
            Err("Invalid duration")
        }
    }
}
impl DurationContent for DateLike {
    fn type_name() -> String {
        "Date".to_owned()
    }
}
impl Display for DateLike {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.into_iso_duration(), f)
    }
}
impl FromStr for DateLike {
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
            expected: PossiblyInfinite<Duration<DateLike>>,
        ) -> Result<(), serde_json::Error> {
            let value = json!(str);
            let duration: PossiblyInfinite<Duration<DateLike>> = serde_json::from_value(value)?;
            assert_eq!(duration, expected, "{str}");
            Ok(())
        }
        test("infinite", PossiblyInfinite::Infinite)?;
        test("P2D", PossiblyInfinite::Finite(Duration(DateLike::Days(2))))?;
        test(
            "P3W",
            PossiblyInfinite::Finite(Duration(DateLike::Weeks(3))),
        )?;
        test(
            "P4M",
            PossiblyInfinite::Finite(Duration(DateLike::Months(4))),
        )?;
        test(
            "P5Y",
            PossiblyInfinite::Finite(Duration(DateLike::Years(5))),
        )?;
        Ok(())
    }
}
