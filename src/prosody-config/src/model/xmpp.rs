// prosody-config
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[cfg(feature = "serde")]
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{fmt::Display, str::FromStr};

// ===== JID =====

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(SerializeDisplay, DeserializeFromStr))]
pub struct JID {
    pub node: String,
    pub domain: String,
}

impl JID {
    pub fn new<S1: ToString, S2: ToString>(node: S1, domain: S2) -> Self {
        Self {
            node: node.to_string(),
            domain: domain.to_string(),
        }
    }
}

impl Display for JID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.node, self.domain)
    }
}

impl FromStr for JID {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once("@") {
            Some((node, domain)) => Ok(Self::new(node, domain)),
            None => Err("The JID does not contain a '@'"),
        }
    }
}
