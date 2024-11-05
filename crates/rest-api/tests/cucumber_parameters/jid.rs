// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Display, ops::Deref, str::FromStr};

use cucumber::Parameter;
use service::model::BareJid;

#[derive(Debug, Parameter)]
#[param(name = "jid", regex = r"[^<]+@[^>]+")]
pub struct JID(pub BareJid);

impl Deref for JID {
    type Target = BareJid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for JID {
    type Err = <BareJid as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BareJid::from_str(s).map(|a| Self(a))
    }
}

impl Display for JID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
