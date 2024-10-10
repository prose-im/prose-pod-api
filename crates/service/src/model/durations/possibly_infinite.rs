// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use sea_orm::{entity::prelude::*, sea_query};

const DURATION_INFINITE: &'static str = "infinite";

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

impl<D: Display> serde::Serialize for PossiblyInfinite<D> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl<'de, Duration: FromStr> serde::Deserialize<'de> for PossiblyInfinite<Duration>
where
    Duration::Err: Display,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
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
