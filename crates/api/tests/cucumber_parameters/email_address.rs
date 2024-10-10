// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::model::EmailAddress as EmailAddressEntityModel;
use std::{fmt::Display, ops::Deref, str::FromStr};

use cucumber::Parameter;

#[derive(Debug, Parameter)]
#[param(name = "email", regex = r"[^<]+@[^>]+")]
pub struct EmailAddress(pub EmailAddressEntityModel);

impl Deref for EmailAddress {
    type Target = EmailAddressEntityModel;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for EmailAddress {
    type Err = <EmailAddressEntityModel as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EmailAddressEntityModel::from_str(s).map(|a| Self(a))
    }
}

impl Display for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
