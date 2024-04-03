// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::model;
use std::{fmt::Display, ops::Deref, str::FromStr};

use cucumber::Parameter;

#[derive(Debug, Parameter)]
#[param(name = "jid", regex = r"[^<]+@[^>]+")]
pub struct JID(pub model::JID);

impl Deref for JID {
    type Target = model::JID;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for JID {
    type Err = <model::JID as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        model::JID::from_str(s).map(|a| Self(a))
    }
}

impl Display for JID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
