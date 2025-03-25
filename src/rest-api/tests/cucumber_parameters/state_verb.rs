// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use cucumber::Parameter;

#[derive(Debug, Parameter)]
#[param(
    name = "state_verb",
    regex = "(is|isn't|is not)|(has|doesn't have|does not have)"
)]
pub struct StateVerb(bool);

impl FromStr for StateVerb {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "is" | "has" => Ok(Self(true)),
            "isn't" | "is not" | "doesn't have" | "does not have" => Ok(Self(false)),
            s => Err(format!("Invalid `StateVerb`: {s}")),
        }
    }
}

impl StateVerb {
    #[inline]
    pub fn into_bool(&self) -> bool {
        self.0
    }
    #[inline]
    pub fn as_bool(&self) -> &bool {
        &self.0
    }
}

impl AsRef<bool> for StateVerb {
    #[inline]
    fn as_ref(&self) -> &bool {
        self.as_bool()
    }
}
