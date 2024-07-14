// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    fmt::{Debug, Display},
    ops::Deref,
    str::FromStr,
};

use iso8601_duration::Duration as ISODuration;
use sea_orm::{entity::prelude::*, sea_query};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::sea_orm_try_get_by_string;

const DURATION_INFINITE: &'static str = "infinite";

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Duration<Content: DurationContent>(pub Content);

impl<Content: DurationContent> Duration<Content> {
    fn as_iso_duration(&self) -> ISODuration {
        self.0.into()
    }
}

impl<Content: DurationContent> Deref for Duration<Content> {
    type Target = Content;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<Content: DurationContent> Eq for Duration<Content> {}

impl<Content: DurationContent> Display for Duration<Content> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_iso_duration())
    }
}
impl<Content: DurationContent> FromStr for Duration<Content> {
    type Err = <Content as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Content::from_str(s).map(Self)
    }
}

impl<Content: DurationContent> Serialize for Duration<Content> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_iso_duration().serialize(serializer)
    }
}

impl<'de, Content: DurationContent> Deserialize<'de> for Duration<Content>
where
    <Content as TryFrom<ISODuration>>::Error: Debug,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let content: Content = ISODuration::deserialize(deserializer)?
            .try_into()
            .map_err(|e| de::Error::custom(format!("{:?}", e)))?;
        Ok(Self(content))
    }
}

impl<Content: DurationContent> From<Duration<Content>> for sea_query::Value {
    fn from(value: Duration<Content>) -> Self {
        let iso_duration: ISODuration = value.0.into();
        Self::String(Some(Box::new(iso_duration.to_string())))
    }
}

impl<Content: DurationContent> TryFrom<String> for Duration<Content>
where
    Content: TryFrom<ISODuration>,
    <Content as TryFrom<ISODuration>>::Error: Debug,
{
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let iso_duration = ISODuration::parse(&value)
            // Technically, the value is not `null`, but we wouldn't want to unsafely unwrap here.
            .map_err(|e| format!("Could not parse ISO 8601 duration: {:?}", e))?;
        let content = Content::try_from(iso_duration)
            // Technically, the value is not `null`, but we wouldn't want to unsafely unwrap here.
            .map_err(|e| format!("Duration is incorrect: {:?}", e))?;
        Ok(Self(content))
    }
}
impl<Content: DurationContent> TryFrom<ISODuration> for Duration<Content> {
    type Error = <Content as TryFrom<ISODuration>>::Error;

    fn try_from(value: ISODuration) -> Result<Self, Self::Error> {
        Content::try_from(value).map(Self)
    }
}

impl<Content: DurationContent> sea_orm::TryGetable for Duration<Content>
where
    <Content as TryFrom<ISODuration>>::Error: Debug,
    <Content as FromStr>::Err: Debug,
{
    sea_orm_try_get_by_string!();
}

impl<Content: DurationContent> sea_query::ValueType for Duration<Content>
where
    Self: TryFrom<String>,
{
    fn try_from(v: Value) -> Result<Self, sea_query::ValueTypeErr> {
        match v {
            Value::String(Some(value)) => {
                Self::from_str(value.as_str()).map_err(|_| sea_query::ValueTypeErr)
            }
            _ => Err(sea_query::ValueTypeErr),
        }
    }

    fn type_name() -> String {
        Content::type_name()
    }

    fn array_type() -> sea_query::ArrayType {
        sea_query::ArrayType::String
    }

    fn column_type() -> ColumnType {
        ColumnType::string(None)
    }
}

impl<Content: DurationContent> sea_query::Nullable for Duration<Content> {
    fn null() -> Value {
        Value::String(None)
    }
}

pub trait DurationContent: Copy + Eq + Into<ISODuration> + TryFrom<ISODuration> + FromStr {
    fn type_name() -> String;
}

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
}

impl Eq for TimeLike {}
impl Into<ISODuration> for TimeLike {
    fn into(self) -> ISODuration {
        self.into_iso_duration()
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

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Copy, Clone, Debug, PartialEq)]
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

impl<D: Eq> Eq for PossiblyInfinite<D> {}

impl<D: Display> Display for PossiblyInfinite<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Infinite => write!(f, "{DURATION_INFINITE}"),
            Self::Finite(d) => Display::fmt(d, f),
        }
    }
}

impl<D: FromStr> FromStr for PossiblyInfinite<D> {
    type Err = <D as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            DURATION_INFINITE => Ok(Self::Infinite),
            d => D::from_str(d).map(Self::Finite),
        }
    }
}

impl<D: Display> Serialize for PossiblyInfinite<D> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl<'de, Duration: FromStr> Deserialize<'de> for PossiblyInfinite<Duration>
where
    Duration::Err: Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::from_str(String::deserialize(deserializer)?.as_str())
            .map_err(|err| serde::de::Error::custom(&err))
    }
}

impl<D: Into<sea_query::Value>> From<PossiblyInfinite<D>> for sea_query::Value {
    fn from(value: PossiblyInfinite<D>) -> Self {
        match value {
            PossiblyInfinite::Infinite => {
                Self::String(Some(Box::new(DURATION_INFINITE.to_string())))
            }
            PossiblyInfinite::Finite(duration) => duration.into(),
        }
    }
}

impl<D: sea_orm::TryGetable> sea_orm::TryGetable for PossiblyInfinite<D> {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        index: I,
    ) -> Result<Self, sea_orm::TryGetError> {
        // https://github.com/SeaQL/sea-orm/discussions/1176#discussioncomment-4024088
        let value = res
            .try_get_by(index)
            .map_err(sea_orm::TryGetError::DbErr)
            .and_then(|opt: Option<String>| {
                opt.ok_or(sea_orm::TryGetError::Null(format!("{index:?}")))
            })?;
        match value.as_str() {
            DURATION_INFINITE => Ok(Self::Infinite),
            _ => D::try_get_by(res, index).map(Self::Finite),
        }
    }
}

impl<D: sea_query::ValueType> sea_query::ValueType for PossiblyInfinite<D> {
    fn try_from(v: Value) -> Result<Self, sea_query::ValueTypeErr> {
        let value: Option<String> = v.unwrap();
        let Some(value) = value else {
            return Err(sea_query::ValueTypeErr);
        };
        match value.as_str() {
            DURATION_INFINITE => Ok(Self::Infinite),
            _ => D::try_from(Value::String(Some(Box::new(value)))).map(Self::Finite),
        }
    }

    fn type_name() -> String {
        format!("PossiblyInfinite<{}>", D::type_name())
    }

    fn array_type() -> sea_query::ArrayType {
        D::array_type()
    }

    fn column_type() -> ColumnType {
        D::column_type()
    }
}

impl<D: sea_query::Nullable> sea_query::Nullable for PossiblyInfinite<D> {
    fn null() -> Value {
        D::null()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_duration_timelike_deserializing() -> Result<(), serde_json::Error> {
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

    #[test]
    fn test_duration_datelike_deserializing() -> Result<(), serde_json::Error> {
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
