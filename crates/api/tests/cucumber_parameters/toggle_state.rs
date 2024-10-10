// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use cucumber::Parameter;

#[derive(Debug, Parameter)]
#[param(name = "toggle", regex = "on|off|enabled|disabled")]
pub enum ToggleState {
    Enabled,
    Disabled,
}

impl ToggleState {
    pub fn as_bool(&self) -> bool {
        match self {
            Self::Enabled => true,
            Self::Disabled => false,
        }
    }
}

impl Into<bool> for ToggleState {
    fn into(self) -> bool {
        self.as_bool()
    }
}

impl FromStr for ToggleState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "on" | "enabled" => Self::Enabled,
            "off" | "disabled" => Self::Disabled,
            invalid => return Err(format!("Invalid `ToggleState`: {invalid}")),
        })
    }
}
