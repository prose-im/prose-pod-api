// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{ops::Deref, str::FromStr};

use cucumber::Parameter;
use entity::model;

#[derive(Debug, Parameter)]
#[param(name = "member_role", regex = r"\w+")]
pub struct MemberRole(pub model::MemberRole);

impl Deref for MemberRole {
    type Target = model::MemberRole;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for MemberRole {
    type Err = <model::MemberRole as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        model::MemberRole::from_str(s).map(Self)
    }
}
