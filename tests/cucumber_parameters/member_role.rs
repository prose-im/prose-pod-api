// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{ops::Deref, str::FromStr};

use cucumber::Parameter;

#[derive(Debug, Parameter)]
#[param(name = "member_role", regex = r"\w+")]
pub struct MemberRole(pub service::model::MemberRole);

impl Deref for MemberRole {
    type Target = service::model::MemberRole;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for MemberRole {
    type Err = <service::model::MemberRole as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        service::model::MemberRole::from_str(s).map(Self)
    }
}
