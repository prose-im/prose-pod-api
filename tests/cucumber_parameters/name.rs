// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{ops::Deref, str::FromStr};

use cucumber::Parameter;

/// The `{word}` parameter doesn't require quotes, but matches only one word.
/// The `{string}` parameter matches a full string with spaces, but requires quotes.
/// This parameter matches a full string with spaces without requiring quotes.
#[derive(Debug, Parameter)]
#[param(name = "name", regex = r".+")]
pub struct Name(pub String);

impl Deref for Name {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Name {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}
