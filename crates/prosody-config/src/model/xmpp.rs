// prosody-config
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Display;

// ===== JID =====

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
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

impl TryFrom<String> for JID {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.split_once("@") {
            Some((node, domain)) => Ok(Self::new(node, domain)),
            None => Err("The JID does not contain a '@'"),
        }
    }
}
