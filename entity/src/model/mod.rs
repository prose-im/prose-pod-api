// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod server_config;

use iso8601_duration::Duration as ISODuration;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{self, ArrayType, ValueTypeErr};
use sea_orm::TryGetError;
use serde::{de, Deserialize, Serialize};

use std::fmt::{Debug, Display};
use std::ops::Deref;

// ===== JID =====

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct JID {
    pub node: String,
    pub domain: String,
}

impl Display for JID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.node, self.domain)
    }
}

// ===== DURATIONS =====

#[derive(Clone, Debug, PartialEq)]
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
{
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::prelude::QueryResult,
        index: I,
    ) -> Result<Self, TryGetError> {
        let value: String = res.try_get_by(index).map_err(TryGetError::DbErr)?;
        Self::try_from(value)
            // Technically, the value is not `null`, but we wouldn't want to unsafely unwrap here.
            .map_err(|e| TryGetError::Null(format!("{:?}", e)))
    }
}

impl<Content: DurationContent> sea_query::ValueType for Duration<Content>
where
    Self: TryFrom<String>,
{
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::String(Some(value)) => (*value).try_into().map_err(|_| ValueTypeErr),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        Content::type_name()
    }

    fn array_type() -> ArrayType {
        ArrayType::String
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

pub trait DurationContent: Copy + Eq + Into<ISODuration> + TryFrom<ISODuration> {
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
}

impl Eq for TimeLike {}
impl Into<ISODuration> for TimeLike {
    /// NOTE: This method is not correct, as a `u32` can overflow in a `f32`.
    ///   As this situation will probably never happen, it's good enough.
    fn into(self) -> ISODuration {
        match self {
            Self::Seconds(n) => ISODuration::new(0., 0., 0., 0., 0., n as f32),
            Self::Minutes(n) => ISODuration::new(0., 0., 0., 0., n as f32, 0.),
            Self::Hours(n) => ISODuration::new(0., 0., 0., n as f32, 0., 0.),
        }
    }
}
impl TryFrom<ISODuration> for TimeLike {
    type Error = &'static str;

    fn try_from(value: ISODuration) -> Result<Self, Self::Error> {
        if let Some(hours) = value.num_hours() {
            Ok(Self::Hours(hours as u32))
        } else if let Some(minutes) = value.num_minutes() {
            Ok(Self::Minutes(minutes as u32))
        } else if let Some(seconds) = value.num_seconds() {
            Ok(Self::Seconds(seconds as u32))
        } else {
            Err("Invalid duration")
        }
    }
}
impl DurationContent for TimeLike {
    fn type_name() -> String {
        stringify!(Time).to_owned()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DateLike {
    Days(u32),
    Weeks(u32),
    Months(u32),
    Years(u32),
}

impl Eq for DateLike {}
impl Into<ISODuration> for DateLike {
    /// NOTE: This method is not correct, as a `u32` can overflow in a `f32`.
    ///   As this situation will probably never happen, it's good enough.
    fn into(self) -> ISODuration {
        match self {
            Self::Days(n) => ISODuration::new(0., 0., n as f32, 0., 0., 0.),
            Self::Weeks(n) => ISODuration::new(0., 0., (n as f32) * 7., 0., 0., 0.),
            Self::Months(n) => ISODuration::new(0., n as f32, 0., 0., 0., 0.),
            Self::Years(n) => ISODuration::new(n as f32, 0., 0., 0., 0., 0.),
        }
    }
}
impl TryFrom<ISODuration> for DateLike {
    type Error = &'static str;

    /// NOTE: This method is not correct, as a `f32` can overflow in a `u32`.
    ///   As this situation will probably never happen, it's good enough.
    fn try_from(value: ISODuration) -> Result<Self, Self::Error> {
        if let Some(years) = value.num_years() {
            Ok(Self::Years(years as u32))
        } else if let Some(months) = value.num_months() {
            Ok(Self::Months(months as u32))
        } else if let Some(weeks) = value.num_weeks() {
            Ok(Self::Weeks(weeks as u32))
        } else if let Some(days) = value.num_days() {
            Ok(Self::Days(days as u32))
        } else {
            Err("Invalid duration")
        }
    }
}
impl DurationContent for DateLike {
    fn type_name() -> String {
        stringify!(Date).to_owned()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

impl<D: Eq> Eq for PossiblyInfinite<D> {}

impl<D> From<PossiblyInfinite<D>> for sea_query::Value
where
    sea_query::Value: From<D>,
{
    fn from(value: PossiblyInfinite<D>) -> Self {
        match value {
            PossiblyInfinite::Infinite => Self::String(Some(Box::new("infinite".to_string()))),
            PossiblyInfinite::Finite(duration) => Self::from(duration),
        }
    }
}

impl<D: sea_orm::TryGetable> sea_orm::TryGetable for PossiblyInfinite<D> {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::prelude::QueryResult,
        index: I,
    ) -> Result<Self, TryGetError> {
        let value: String = res.try_get_by(index).map_err(TryGetError::DbErr)?;
        match value.as_str() {
            "infinite" => Ok(Self::Infinite),
            _ => D::try_get_by(res, index).map(Self::Finite),
        }
    }
}

impl<D: sea_query::ValueType> sea_query::ValueType for PossiblyInfinite<D> {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        let value: Option<String> = v.unwrap();
        let Some(value) = value else {
            return Err(ValueTypeErr);
        };
        match value.as_str() {
            "infinite" => Ok(Self::Infinite),
            _ => D::try_from(Value::String(Some(Box::new(value)))).map(Self::Finite),
        }
    }

    fn type_name() -> String {
        format!("{}<{}>", stringify!(PossiblyInfinite), D::type_name())
    }

    fn array_type() -> ArrayType {
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