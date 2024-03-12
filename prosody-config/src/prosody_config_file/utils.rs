// prosody-config
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::*;

impl LuaComment {
    pub fn new<S: ToString>(s: S) -> Self {
        Self(s.to_string())
    }
}

impl<T> Group<T> {
    pub fn new<C: Into<LuaComment>>(comment: C, elements: Vec<T>) -> Self {
        Self {
            comment: Some(comment.into()),
            elements,
        }
    }
    pub fn flattened(comment: Option<&str>, elements: Vec<Option<T>>) -> Option<Self> {
        let elements: Vec<T> = elements.into_iter().flatten().collect();
        if elements.is_empty() {
            None
        } else {
            Some(Self {
                comment: comment.map(Into::into),
                elements,
            })
        }
    }
}

impl LuaDefinition {
    pub fn comment<C: Into<LuaComment>>(mut self, comment: C) -> Self {
        self.comment = Some(comment.into());
        self
    }
}

pub fn def<K: ToString, V: Into<LuaValue>>(key: K, value: V) -> LuaDefinition {
    LuaDefinition {
        comment: None,
        key: key.to_string(),
        value: value.into(),
    }
}

pub fn option_def<K: ToString, V: Into<LuaValue>>(
    comment: Option<&str>,
    key: K,
    value: Option<V>,
) -> Option<LuaDefinition> {
    value.map(|value| LuaDefinition {
        comment: comment.map(Into::into),
        key: key.to_string(),
        value: value.into(),
    })
}

pub fn mult<LHS: Into<LuaNumber>, RHS: Into<LuaNumber>>(lhs: LHS, rhs: RHS) -> LuaNumber {
    LuaNumber::Product(Box::new(lhs.into()), Box::new(rhs.into()))
}
