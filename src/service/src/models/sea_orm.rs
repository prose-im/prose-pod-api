// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    fmt::{Debug, Display},
    ops::Deref,
    str::FromStr,
};

// ===== AsString =====

pub struct SeaOrmAsString<T: Display + FromStr>(pub T);

impl<T: Display + FromStr> Display for SeaOrmAsString<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
impl<T: Display + FromStr> FromStr for SeaOrmAsString<T> {
    type Err = <T as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        T::from_str(s).map(Self)
    }
}
impl<T: Display + FromStr + Debug> Debug for SeaOrmAsString<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}
impl<T: Display + FromStr + Clone> Clone for SeaOrmAsString<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: Display + FromStr + PartialEq> PartialEq for SeaOrmAsString<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl<T: Display + FromStr> From<T> for SeaOrmAsString<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: Display + FromStr> Deref for SeaOrmAsString<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Display + FromStr> From<SeaOrmAsString<T>> for sea_orm::sea_query::Value {
    crate::sea_orm_string_value_impl!(SeaOrmAsString<T>);
}
impl<T: Display + FromStr> sea_orm::TryGetable for SeaOrmAsString<T>
where
    <T as FromStr>::Err: Debug,
{
    crate::sea_orm_try_get_by_string!();
}
impl<T: Display + FromStr> sea_orm::sea_query::ValueType for SeaOrmAsString<T> {
    crate::sea_orm_string_value_type_impl!(SeaOrmAsString<T>, None);
}
impl<T: Display + FromStr> sea_orm::sea_query::Nullable for SeaOrmAsString<T> {
    crate::sea_orm_string_nullable_impl!();
}

// ===== LinkedStringSet =====

#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(sea_orm::FromJsonQueryResult)]
#[repr(transparent)]
pub struct LinkedStringSet(pub linked_hash_set::LinkedHashSet<String>);

impl std::ops::Deref for LinkedStringSet {
    type Target = linked_hash_set::LinkedHashSet<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<linked_hash_set::LinkedHashSet<String>> for LinkedStringSet {
    fn from(value: linked_hash_set::LinkedHashSet<String>) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for LinkedStringSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        let len = self.len();
        for (i, s) in self.0.iter().enumerate() {
            write!(f, "{s:?}")?;
            if i < len - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "}}")
    }
}
