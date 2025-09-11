// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{ops::Deref, str::FromStr};

use cucumber::Parameter;
use service::network_checks::DnsRecordDiscriminants;

#[derive(Debug, Parameter)]
#[repr(transparent)]
#[param(name = "dns_record_type", regex = r"[A-Z]+")]
pub struct DnsRecordType(pub DnsRecordDiscriminants);

impl Deref for DnsRecordType {
    type Target = DnsRecordDiscriminants;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for DnsRecordType {
    type Err = <DnsRecordDiscriminants as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        DnsRecordDiscriminants::from_str(s).map(Self)
    }
}
