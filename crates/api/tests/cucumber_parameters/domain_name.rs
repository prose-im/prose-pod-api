// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{ops::Deref, str::FromStr};

use cucumber::Parameter;
use hickory_proto::rr::Name as HickoryDomainName;

#[derive(Debug, Parameter)]
#[param(name = "domain_name", regex = r"(?:(?:\w|-)+\.)*(?:\w|-)+\.?")]
pub struct DomainName(pub HickoryDomainName);

impl Deref for DomainName {
    type Target = HickoryDomainName;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<HickoryDomainName> for DomainName {
    fn into(self) -> HickoryDomainName {
        self.0
    }
}

impl FromStr for DomainName {
    type Err = <HickoryDomainName as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        HickoryDomainName::from_str(s).map(Self)
    }
}

impl PartialEq<DomainName> for HickoryDomainName {
    fn eq(&self, other: &DomainName) -> bool {
        self.eq(&other.0)
    }
}
