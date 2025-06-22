// prosody-config
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod print;
pub mod utils;

use linked_hash_map::LinkedHashMap;
use linked_hash_set::LinkedHashSet;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{hash::Hash, path::PathBuf};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProsodyConfigFile {
    pub header: Option<Group<LuaComment>>,
    pub global_settings: Vec<Group<LuaDefinition>>,
    pub additional_sections: Vec<ProsodyConfigFileSection>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ProsodyConfigFileSection {
    VirtualHost {
        comments: Vec<LuaComment>,
        hostname: String,
        settings: Vec<Group<LuaDefinition>>,
    },
    Component {
        comments: Vec<LuaComment>,
        hostname: String,
        plugin: String,
        name: String,
        settings: Vec<Group<LuaDefinition>>,
    },
}

// ===== Atoms =====

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LuaComment(pub String);

impl<S: ToString> From<S> for LuaComment {
    fn from(value: S) -> Self {
        Self(value.to_string())
    }
}

/// When we want to group definitions together by topic for example,
/// we can use groups to avoid printing empty lines in-between.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Group<T> {
    #[cfg_attr(feature = "serde", serde(skip))]
    pub comment: Option<LuaComment>,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub elements: Vec<T>,
}

impl<T> Group<T> {
    pub fn empty() -> Self {
        Self {
            comment: None,
            elements: vec![],
        }
    }
}

impl<T> From<T> for Group<T> {
    fn from(value: T) -> Self {
        Self {
            comment: None,
            elements: vec![value],
        }
    }
}

impl<T> From<Vec<T>> for Group<T> {
    fn from(value: Vec<T>) -> Self {
        Self {
            comment: None,
            elements: value,
        }
    }
}

impl LuaDefinition {
    pub fn as_group(self) -> Group<Self> {
        self.into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LuaDefinition {
    pub comment: Option<LuaComment>,
    pub key: String,
    pub value: LuaValue,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LuaNumber {
    Scalar(i64),
    Product(Box<LuaNumber>, Box<LuaNumber>),
}

impl From<i64> for LuaNumber {
    fn from(value: i64) -> Self {
        LuaNumber::Scalar(value)
    }
}

impl From<i16> for LuaNumber {
    fn from(value: i16) -> Self {
        i64::from(value).into()
    }
}

impl From<u8> for LuaNumber {
    fn from(value: u8) -> Self {
        i64::from(value).into()
    }
}

impl From<u16> for LuaNumber {
    fn from(value: u16) -> Self {
        i64::from(value).into()
    }
}

impl From<i32> for LuaNumber {
    fn from(value: i32) -> Self {
        LuaNumber::Scalar(i64::from(value))
    }
}

impl From<u32> for LuaNumber {
    fn from(value: u32) -> Self {
        LuaNumber::Scalar(i64::from(value))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LuaValue {
    Bool(bool),
    Number(LuaNumber),
    String(String),
    List(Vec<LuaValue>),
    Map(LinkedHashMap<String, LuaValue>),
}

impl LuaValue {
    pub fn is_scalar(&self) -> bool {
        match self {
            Self::Bool(_) | Self::Number(_) | Self::String(_) => true,
            Self::List(_) | Self::Map(_) => false,
        }
    }
}

impl From<bool> for LuaValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl<T: Into<LuaNumber>> From<T> for LuaValue {
    fn from(value: T) -> Self {
        LuaValue::Number(value.into())
    }
}

impl From<String> for LuaValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&String> for LuaValue {
    fn from(value: &String) -> Self {
        Self::String(value.clone())
    }
}

impl From<PathBuf> for LuaValue {
    fn from(value: PathBuf) -> Self {
        Self::String(value.display().to_string())
    }
}

impl From<&str> for LuaValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl<V: Into<LuaValue>> From<Vec<V>> for LuaValue {
    fn from(value: Vec<V>) -> Self {
        Self::List(value.into_iter().map(Into::into).collect())
    }
}

impl<V: Into<LuaValue> + Eq + Hash> From<LinkedHashSet<V>> for LuaValue {
    fn from(value: LinkedHashSet<V>) -> Self {
        Self::List(value.into_iter().map(Into::into).collect())
    }
}

impl<K, V> Into<LuaValue> for LinkedHashMap<K, V>
where
    K: Into<String> + Hash + Eq,
    V: Into<LuaValue>,
{
    fn into(self) -> LuaValue {
        let mut map = LinkedHashMap::<String, LuaValue>::new();
        for (k, v) in self {
            map.insert(k.into(), v.into());
        }
        LuaValue::Map(map)
    }
}

impl<K, V> Into<LuaValue> for Vec<(K, V)>
where
    K: Into<String> + Hash + Eq,
    V: Into<LuaValue>,
{
    fn into(self) -> LuaValue {
        let map: LinkedHashMap<K, V> = self.into_iter().collect();
        map.into()
    }
}
