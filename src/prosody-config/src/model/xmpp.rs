// prosody-config
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[cfg(feature = "serde")]
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{convert::Infallible, fmt::Display, str::FromStr};

// ===== JID =====

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(SerializeDisplay, DeserializeFromStr))]
pub struct BareJid {
    pub node: Option<String>,
    pub domain: String,
}

impl BareJid {
    pub fn new(node: impl ToString, domain: impl ToString) -> Self {
        Self {
            node: Some(node.to_string()),
            domain: domain.to_string(),
        }
    }
    pub fn domain(domain: impl ToString) -> Self {
        Self {
            node: None,
            domain: domain.to_string(),
        }
    }
}

impl Display for BareJid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.node.as_ref() {
            Some(node) => write!(f, "{}@{}", node, self.domain),
            None => write!(f, "{}", self.domain),
        }
    }
}

impl FromStr for BareJid {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once("@") {
            Some((node, domain)) => Ok(Self::new(node, domain)),
            None => Ok(Self::domain(s)),
        }
    }
}
