// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod date_like;
mod possibly_infinite;
mod time_like;

use std::{
    fmt::{Debug, Display},
    ops::Deref,
    str::FromStr,
};

use iso8601_duration::Duration as ISODuration;

pub use date_like::*;
pub use possibly_infinite::*;
use sea_orm::sea_query;
pub use time_like::*;

use crate::sea_orm_try_get_by_string;

// TODO: Use `iso8601_duration` only as a (de)serializer,
//   and use `time::Duration` in domain models.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Duration<Content: DurationContent>(pub Content);

pub trait DurationContent: Copy + Eq + Into<ISODuration> + TryFrom<ISODuration> + FromStr {
    fn type_name() -> String;
}

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

impl<Content: DurationContent> serdev::Serialize for Duration<Content> {
    fn serialize<S: serdev::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_iso_duration().serialize(serializer)
    }
}

impl<'de, Content: DurationContent> serdev::Deserialize<'de> for Duration<Content>
where
    <Content as TryFrom<ISODuration>>::Error: Debug,
{
    fn deserialize<D: serdev::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Content::try_from(ISODuration::deserialize(deserializer)?)
            .map_err(|err| serdev::de::Error::custom(format!("{err:?}")))
            .map(Self)
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
            .map_err(|e| format!("Could not parse ISO 8601 duration: {e:?}"))?;
        let content = Content::try_from(iso_duration)
            // Technically, the value is not `null`, but we wouldn't want to unsafely unwrap here.
            .map_err(|e| format!("Duration is incorrect: {e:?}"))?;
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
    sea_orm_try_get_by_string!(using: FromStr);
}

impl<Content: DurationContent> sea_query::ValueType for Duration<Content>
where
    Self: TryFrom<String>,
{
    fn try_from(v: sea_orm::Value) -> Result<Self, sea_query::ValueTypeErr> {
        match v {
            sea_orm::Value::String(Some(value)) => {
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

    fn column_type() -> sea_orm::ColumnType {
        sea_orm::ColumnType::string(None)
    }
}

impl<Content: DurationContent> sea_query::Nullable for Duration<Content> {
    fn null() -> sea_orm::Value {
        sea_orm::Value::String(None)
    }
}
