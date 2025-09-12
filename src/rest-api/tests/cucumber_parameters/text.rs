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
#[repr(transparent)]
#[param(name = "text", regex = r".+")]
pub struct Text(pub String);

impl Deref for Text {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Text {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}
