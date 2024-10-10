// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use cucumber::Parameter;

#[derive(Debug, Parameter)]
#[param(name = "open_or_not", regex = "open|closed|not open")]
pub enum OpenState {
    Open,
    Closed,
}

impl OpenState {
    pub fn as_bool(&self) -> bool {
        match self {
            Self::Open => true,
            Self::Closed => false,
        }
    }
}

impl Into<bool> for OpenState {
    fn into(self) -> bool {
        self.as_bool()
    }
}

impl FromStr for OpenState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "open" => Self::Open,
            "closed" | "not open" => Self::Closed,
            invalid => return Err(format!("Invalid `OpenState`: {invalid}")),
        })
    }
}
