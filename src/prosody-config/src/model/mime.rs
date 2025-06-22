// prosody-config
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)
)]
pub struct Mime(pub mime::Mime);

impl Display for Mime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl FromStr for Mime {
    type Err = <mime::Mime as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        mime::Mime::from_str(s).map(Self)
    }
}
