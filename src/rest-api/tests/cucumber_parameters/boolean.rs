// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{ops::Deref, str::FromStr};

use cucumber::Parameter;

#[derive(Debug, Parameter)]
#[param(name = "bool", regex = "true|false")]
pub struct Bool(bool);

impl Deref for Bool {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Bool {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "true" => Self(true),
            "false" => Self(false),
            invalid => return Err(format!("Invalid `Bool`: {invalid}")),
        })
    }
}
